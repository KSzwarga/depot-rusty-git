use std::fs::Metadata;
use std::fs::OpenOptions;
use std::fs::File;
use std::fs::rename;
use std::io;
use std::io::Write;
use sha1::{Sha1, Digest};   
use flate2::{write::ZlibEncoder, Compression};
use memmap::Mmap;
use std::time::{SystemTime};

use crate::model::cache::CacheEntry;
use crate::model::cache_header::CacheHeader;
use crate::commands::read_cache::read_cache;
use crate::constants::DB_ENVIRONMENT;
use crate::constants::HEX;

pub fn build_sha1_path(sha1: &[u8; 20]) -> String {
    // 40 hex chars + 1 slash + length of DB_ENVIRONMENT
    let capacity = DB_ENVIRONMENT.len() + 41;
    let mut buf = Vec::with_capacity(capacity);

    buf.extend_from_slice(DB_ENVIRONMENT.as_bytes());
    buf.push(HEX[(sha1[0] >> 4) as usize]);
    buf.push(HEX[(sha1[0] & 0xf) as usize]);
    buf.push(b'/'); 

    for &byte in &sha1[1..20] {
        buf.push(HEX[(byte >> 4) as usize]);
        buf.push(HEX[(byte & 0xf) as usize]);
    }

    String::from_utf8_lossy(&buf).into_owned()
}

pub fn write_sha1_file(content: &Vec<u8>) -> Result<[u8; 20], std::io::Error> {
    let mut encoder = ZlibEncoder::new(Vec::with_capacity(content.len() + 200), Compression::best());
    encoder.write_all(&content)?;
    let compressed = encoder.finish()?;

    let sha1: [u8; 20] = Sha1::digest(&compressed).into();
    let path = build_sha1_path(&sha1);
    match OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
        {
            Ok(mut f) => {
                f.write_all(&compressed)?},
            Err(e) if e.kind() == io::ErrorKind::AlreadyExists => (),
            Err(e) => {
                eprintln!("Unexpected error when creating sha1 file: {}", e);
            }
        }
    Ok(sha1)
}

pub fn create_blob(file: File, metadata: Metadata) -> Result<[u8; 20], std::io::Error>{
    let file_len = metadata.len() as usize;
    let mmap;
    let mut content: Vec<u8>;
    if file_len == 0 {
        content = Vec::new();
    } else {
        mmap = unsafe { Mmap::map(&file) }?;
        content = format!("blob {}\0", file_len).into();
        content.extend_from_slice(&mmap);
    }
    drop(file);
    let sha1 = write_sha1_file(&content)?;
    Ok(sha1)
}

pub fn add_file_to_cache(path: &str, active_cache:&mut Vec<CacheEntry>)->Result<(), std::io::Error>  {
    let file = match File::open(path) {
        Ok(f) => f,
        Err(e) => {
            if e.kind() == io::ErrorKind::NotFound {
                if remove_cache_entry(path, active_cache) == 1 {
                    eprintln!("file not found: {},  file found in cache, removing file from cache: {}", path, e);
                    return Err(e.into());
                } else {
                    eprintln!("file not found, in location or cache: {},  {}, fatal error", path, e);
                    std::fs::remove_file(".depot/index.lock").ok();
                    std::process::exit(1)}
            } else {
                eprintln!("Unexpected error:{}, error: {}", path, e);
                return Err(e.into());
            }
        }
    };

    let metadata = match file.metadata() {
        Ok(m) => m,
        Err(e) => {
            eprintln!("Unexpected error:{}, error: {}", path, e);
            return Err(e.into());
        },
    };

    let name = path.to_string();
    let mut ce = CacheEntry {
        created: metadata.created().unwrap().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_nanos(),
        modified: metadata.modified().unwrap().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_nanos(),
        size: metadata.len(),
        sha1: [0u8; 20],
        namelen: name.len() as u16,
        name,
    };
    ce.sha1 = create_blob(file, metadata)?;
    add_cache_entry(ce, active_cache);

    Ok(())
}

pub fn add_cache_entry(ce: CacheEntry, active_cache: &mut Vec<CacheEntry>) {
    let pos = cache_name_pos(&ce.name, active_cache);
    if pos < 0 {
        let idx = (-pos - 1) as usize;
        active_cache[idx] = ce;         
    } else {
        active_cache.insert(pos as usize, ce); 
    }
}

pub fn remove_cache_entry(name: &str , active_cache: &mut Vec<CacheEntry>) -> isize {
    let pos = cache_name_pos(name, active_cache);
    if pos < 0 {
        let idx = (-pos - 1) as usize;
        println!("{}",idx);
        active_cache.remove(idx);
        return 1;
    }
    return 0;
}

fn cache_name_pos(name: &str, active_cache: &[CacheEntry]) -> isize {
    let mut first = 0isize;
    let mut last = active_cache.len() as isize;

    while last > first {
        let next = (last + first) >> 1;
        let ce = &active_cache[next as usize];

        match name.cmp(&ce.name) {
            std::cmp::Ordering::Equal   => return -next - 1,
            std::cmp::Ordering::Less    => last = next,
            std::cmp::Ordering::Greater => first = next + 1,
        }
    }

    first
}

pub fn write_cache(mut file: File, active_cache: &[CacheEntry], nr_entries: i32)  -> Result<(), std::io::Error> {
    let mut hdr = CacheHeader {
        signature: 0x44495243u32, //DIRC
        version: 1,
        nr_entries: nr_entries,
        sha1: [0u8; 20]
    };
    let mut hasher = Sha1::new() ;

    hasher.update(&hdr.signature.to_be_bytes());
    hasher.update(&hdr.version.to_be_bytes());
    hasher.update(&hdr.nr_entries.to_be_bytes());
    for ce in active_cache {
        hasher.update(&ce.created.to_be_bytes());
        hasher.update(&ce.modified.to_be_bytes());
        hasher.update(&ce.size.to_be_bytes());
        hasher.update(&ce.sha1);
        hasher.update(&ce.namelen.to_be_bytes());
        hasher.update(&ce.name.as_bytes());
    }
    hdr.sha1 = hasher.finalize().into();

    file.write_all(&hdr.signature.to_be_bytes())?;
    file.write_all(&hdr.version.to_be_bytes())?;
    file.write_all(&hdr.nr_entries.to_be_bytes())?;
    file.write_all(&hdr.sha1)?;
    for ce in active_cache {
        file.write_all(&ce.created.to_be_bytes())?;
        file.write_all(&ce.modified.to_be_bytes())?;
        file.write_all(&ce.size.to_be_bytes())?;
        file.write_all(&ce.sha1)?;
        file.write_all(&ce.namelen.to_be_bytes())?;
        file.write_all(&ce.name.as_bytes())?;
    }
    Ok(())
}


pub fn run(args: &[String]) {
    let mut active_cache: Vec<CacheEntry> = match read_cache() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Reading cache failure: {}", e);
                                    std::process::exit(1);},
    };
    let paths = &args[2..]; 
    let num_paths = paths.len();
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create_new(true)
        .open(".depot/index.lock")
        .unwrap();

    for i in 0..num_paths {
        match add_file_to_cache(&paths[i], &mut active_cache){
            Ok(_) => (),
            Err(e) =>
            if e.kind() == io::ErrorKind::NotFound {()}
            else {eprintln!("Adding file failed: {}", e);
            std::fs::remove_file(".depot/index.lock").ok();
            std::process::exit(1);},
            
        };
    }   
    match write_cache(file, &active_cache, active_cache.len() as i32) {
    Ok(_) => {
        if let Err(e) = rename(".depot/index.lock", ".depot/index") {
            eprintln!("Rename failed: {}", e);
            std::fs::remove_file(".depot/index.lock").ok();
            std::process::exit(1);
        }
    },
    Err(e) => {
        eprintln!("Write cache failed: {}", e);
        std::fs::remove_file(".depot/index.lock").ok();
        std::process::exit(1);
    }
}
   }
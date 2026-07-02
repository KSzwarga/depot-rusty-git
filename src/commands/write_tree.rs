use std::{fs::exists, io::{BufWriter, Error, Write}, mem};
use crate::{commands::{add::build_sha1_path, read_cache::read_cache}, model::cache::CacheEntry};

fn check_valid_sha1(sha1: &[u8; 20]) -> Result<bool, Error>{
    let path = build_sha1_path(sha1);
    exists(path)
}

pub fn run() {
    let active_cache: Vec<CacheEntry> = match read_cache() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Reading cache failure: {}", e);
                                    std::process::exit(1);},
    };

    println!("{:?}", active_cache);
    let offset: usize = 40;
    let size =  offset + active_cache.len()*40 + 400;
    let mut buffer = vec![0u8; offset];
    
    for entry in active_cache {
        match check_valid_sha1(&entry.sha1) {
            Ok(true) => (),
            Ok(false) => {
                eprintln!("Error: blob file doesn't exist for entry {:?}", entry);
                std::process::exit(1);
            }
            Err(e) => {
                eprintln!("Error checking sha1 for entry {:?}: {}", entry, e);
                std::process::exit(1);
            }
        }
        buffer.extend_from_slice(&entry.size.to_be_bytes());
        buffer.extend_from_slice(entry.name.as_bytes());
        buffer.push(b'\0');
    }

    
}
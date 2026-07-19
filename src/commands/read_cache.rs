use std::{error::Error, fs::{File, OpenOptions}, io};

use memmap::Mmap;
use sha1::{Digest, Sha1};

use crate::model::cache::CacheEntry;

const SHA1_OFFSET: usize = 12;

fn verify_header(data: &[u8], sha1_offset: usize) -> Result<(), std::io::Error> {
    let stored_sha1 = &data[sha1_offset..sha1_offset + 20];

    let mut hasher = Sha1::new();
    hasher.update(&data[..sha1_offset]);  // header fields before sha1
    hasher.update(&data[sha1_offset + 20..]); // everything after sha1
    
    let result: [u8; 20] = hasher.finalize().into();

    if result != stored_sha1 {
        return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "header validation failed"));
    }
    Ok(())
}

pub fn read_cache() -> Result<Vec<CacheEntry>, Box<dyn Error>>{
    let file: File = match OpenOptions::new()
        .read(true)
        .open(".depot/index") {
            Ok(f) => f,
            Err(e) => {
                if e.kind() == io::ErrorKind::NotFound {
                     return Ok(Vec::new());
                } else {
                eprintln!("Unexpected error reading cache: {}", e);
                return Err(Box::new(e));
        }}};

    let mmap: Mmap = unsafe { Mmap::map(&file) }?;
    verify_header(&mmap, SHA1_OFFSET)?;
    let nr_entries = i32::from_be_bytes(mmap[8..12].try_into()?);

    let mut active_cache: Vec<CacheEntry> = Vec::new();
    let mut curr_entry: usize = SHA1_OFFSET + 20;
    
    for _ in 0..nr_entries {
        let namelen = u16::from_be_bytes(mmap[curr_entry+60..curr_entry+62].try_into()?) as usize;
        let ce: CacheEntry = CacheEntry {
            created: u128::from_be_bytes(mmap[curr_entry..curr_entry+16].try_into()?),
            modified: u128::from_be_bytes(mmap[curr_entry+16..curr_entry+32].try_into()?),
            size: u64::from_be_bytes(mmap[curr_entry+32..curr_entry+40].try_into()?),
            sha1: mmap[curr_entry+40..curr_entry+60].try_into()?,
            namelen: namelen as u16,
            name: String::from_utf8(mmap[curr_entry+62..curr_entry+62+namelen].to_vec())?,
        };
        curr_entry += 62+namelen;
        active_cache.push(ce);
        
    }
    
    Ok(active_cache)
}

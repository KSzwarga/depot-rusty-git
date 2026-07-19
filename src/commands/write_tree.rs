use std::{fs::exists, io::{Error}};
use crate::{commands::{add::{build_sha1_path, write_sha1_file}, read_cache::read_cache}, model::cache::CacheEntry};

fn check_valid_sha1(sha1: &[u8; 20]) -> Result<bool, Error>{
    let path = build_sha1_path(sha1);
    exists(path)
}

pub fn prepend_integer(buf: &mut Vec<u8>, mut tree_size: usize, mut offset: usize) -> usize {
    if tree_size == 0 {
        offset -= 1;
        buf[offset] = b'0';
        return offset;
    }

    while tree_size > 0 {
        offset -= 1;
        let rest = (tree_size % 10) as u8;
        tree_size /= 10;
        buf[offset] = b'0' + rest;
    }
    offset
}

pub fn run() {
    let active_cache: Vec<CacheEntry> = match read_cache() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Reading cache failure: {}", e);
                                    std::process::exit(1);},
    };

    let mut offset: usize = 40;
    let mut buffer = vec![0u8; offset+1];
    
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
        buffer.extend_from_slice(&entry.size.to_string().as_bytes());
        buffer.push(b' ');
        buffer.extend_from_slice(entry.name.as_bytes());
        buffer.push(b'\0');
        buffer.extend_from_slice(&entry.sha1);
    }
    let tree_size = buffer.len() - offset;
    offset = prepend_integer(&mut buffer, tree_size, offset);
    buffer.splice(0..offset, b"tree ".to_vec());
    let sha1 = match write_sha1_file(&buffer){
            Ok(sha) => sha,
            Err(e) => {
                eprintln!("Error checking write_sha1_file for tree: {}", e);
                std::process::exit(1);
            }
        };
    println!("tree written to: {:?}", build_sha1_path(&sha1));
}
use std::io::{BufRead, BufReader, Read};

use crate::commands::show_diff::read_sha1_file;
use crate::constants::HEX;

fn read_tree(sha1: &[u8; 20]) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let tree_contents = read_sha1_file(sha1, "tree")?;
    let mut reader = BufReader::new(tree_contents.as_slice());
    loop {
        let mut line= Vec::new();
        let curr= reader.read_until(b'\0', &mut line)?;
        if curr == 0 {
            break;
        }
        let mut sha1 = [0u8; 20];
        reader.read_exact(&mut sha1)?;
        println!("{} {}", String::from_utf8_lossy(&line), sha1_to_hex(&sha1));
    }
     Ok(tree_contents)
}

pub fn hex_to_sha1(hex: &str) -> Result<[u8; 20], std::num::ParseIntError> {
    let mut sha1 = [0u8; 20];
    for (i, byte) in (0..40)
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16))
        .enumerate()
        {
            sha1[i] = byte?;
        }
    Ok(sha1)
}

pub fn sha1_to_hex(sha1: &[u8; 20]) -> String {
    let mut buf = Vec::with_capacity(40);
    for &byte in &sha1[0..20] {
        buf.push(HEX[(byte >> 4) as usize]);
        buf.push(HEX[(byte & 0xf) as usize]);
    }
    String::from_utf8_lossy(&buf).into_owned()
}

pub fn run(args: &[String]) {
    let arg = &args[2];
    let sha1 = match hex_to_sha1(&arg) {
        Ok(sha1) => sha1,
        Err(e) => 
            {
            eprintln!("Incorrect sha1: {}", e);
            std::process::exit(1);
            }
    };
    match read_tree(&sha1) {
        Ok(sha1) => sha1,
        Err(e) => 
            {
            eprintln!("Error reading tree: {}", e);
            std::process::exit(1);
            }
    };
}
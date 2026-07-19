use std::fs;
use std::path::PathBuf;

use crate::constants::DB_ENVIRONMENT;

pub fn run() {
    match fs::create_dir_all(DB_ENVIRONMENT) {
        Ok(_) => {},
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {},
        Err(e) => eprintln!("Failed to create bucket: {}", e), 
    }       

    for i in 0..256 {
        let path = PathBuf::from(DB_ENVIRONMENT)
            .join(format!("{:02x}", i));
        match fs::create_dir(path)  {
            Ok(_) => {},
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {},
            Err(e) => eprintln!("Failed to create bucket: {}", e),
        }
    }
}

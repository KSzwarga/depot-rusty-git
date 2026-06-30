use std::{fs::{File, Metadata}, io::{self, BufRead, BufReader, BufWriter, Read}, time::SystemTime};
use flate2::read::ZlibDecoder;
use similar::{TextDiff};
use crate::{commands::{add::build_sha1_path, read_cache::read_cache}, model::cache::CacheEntry};

const MODIFIED: i8 = 1 << 0;
const CREATED: i8 = 1 << 1;
const SIZE: i8 = 1 << 2;

fn read_sha1_file(sha1: &[u8; 20]) -> Result<String, Box<dyn std::error::Error>>  {
    let filename = build_sha1_path(sha1);
    let file = File::open(&filename)?;
    let _metadata = &file.metadata()?;
    let decoder = ZlibDecoder::new(file);
    let mut reader = BufReader::new(decoder);
    let _ = reader.read_until(b'\0', &mut Vec::new())?;
    let mut s = String::new();
    reader.read_to_string(&mut s)?;
    Ok(s)
}

fn diff_metadata(ce: &CacheEntry, old_metadata: &Metadata) -> i8 {
    let mut flags: i8 = 0;
    if old_metadata.created().unwrap().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_nanos() != ce.created {flags |= CREATED}
    if old_metadata.modified().unwrap().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_nanos() != ce.modified {flags |= MODIFIED}
    if old_metadata.len() != ce.size {flags |= SIZE}
    return flags;
}

pub fn run() {
    let active_cache: Vec<CacheEntry> = match read_cache() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Reading cache failure: {}", e);
                                    std::process::exit(1);},
    };

    if active_cache.len() == 0 {
        eprint!("cache is empty");
    }

    let mut out = BufWriter::new(io::stdout());

    for ce in active_cache {
        let path = &ce.name;
        let file = match File::open(path) {
            Ok(f) => f,
            Err(e) => {
                    eprintln!("Unexpected error on file opening:{}, error: {}", path, e);
                    continue;
                }
        };
        let old_metadata = match file.metadata() {
            Ok(m) => m,
            Err(e) => {
                eprintln!("Unexpected error on metadata:{}, error: {}", path, e);
                continue;
            },
        };
        let changed: i8 = diff_metadata(&ce, &old_metadata);

        if changed == 0 {
            print!("file: {} unchanged", ce.name);
            continue;
        }

        let old_file = match read_sha1_file(&ce.sha1){
            Ok(m) => m,
            Err(e) => {
                eprintln!("Unexpected error on reading old file:{}, error: {}", path, e);
                continue;
            },
        };
        let new_file = match std::fs::read_to_string(path) {
            Ok(m) => m,
            Err(e) => {
                eprintln!("Unexpected error on reading new file:{}, error: {}", path, e);
                continue;
            },
        };
        TextDiff::from_lines(&old_file, &new_file)
            .unified_diff()
            .to_writer(&mut out)
            .unwrap();
    }
}
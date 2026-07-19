use crate::{commands::{add::{build_sha1_path, write_sha1_file}, read_tree::hex_to_sha1, write_tree::prepend_integer}, constants::{AUTHOR_EMAIL, AUTHOR_NAME, AUTHOR_SURNAME, COMMITER_EMAIL, COMMITER_NAME, COMMITER_SURNAME}};

pub fn run(args: &[String]){
    if hex_to_sha1(&args[2]).is_err() {
        eprintln!("Incorrect child tree");
        std::process::exit(1);
    }
    let tree_sha1= &args[2];
    let mut parent_sha1: Vec<&String> = Vec::new();
    let mut comment = "Empty comment";

    let mut iter = args[3..].iter();
    while let Some(arg) = iter.next() {
        if arg == "-p" {
            let sha = iter.next().expect("expected sha1 after -p");
            if hex_to_sha1(sha).is_err() {
                eprintln!("Incorrect parent tree");
                std::process::exit(1);
            }
            parent_sha1.push(sha);
        } else if arg == "-m" {
            comment = iter.next().expect("expected comment after -m");
        } else {
            eprintln!("Incorrect arguments, correct flags are -p and -m");
            std::process::exit(1);
        }
    }

    let mut offset: usize = 40;
    let mut buffer = vec![0u8; offset+1];

    buffer.extend_from_slice(&format!("tree: {:?}\n", tree_sha1).as_bytes());
    for parent in parent_sha1 {
        buffer.extend_from_slice(&format!("parent: {:?}\n", parent).as_bytes());
    }
    
    //A placeholder for logic to derive the below
    buffer.extend_from_slice(&format!("author: {} {} {}\n", AUTHOR_EMAIL, AUTHOR_NAME, AUTHOR_SURNAME).as_bytes());
    buffer.extend_from_slice(&format!("commiter: {} {} {}\n\n", COMMITER_EMAIL, COMMITER_NAME, COMMITER_SURNAME).as_bytes());
    buffer.extend_from_slice(comment.as_bytes());

    let commit_size = buffer.len() - offset;
    offset = prepend_integer(&mut buffer, commit_size, offset);
    buffer.splice(0..offset, b"commit ".to_vec());


    match write_sha1_file(&buffer){
        Ok(sha) => println!("tree written to: {:?}", build_sha1_path(&sha)),
        Err(e) => {
            eprintln!("Error checking write_sha1_file for commit: {}", e);
            std::process::exit(1);
        }
    };
}
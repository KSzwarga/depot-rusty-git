mod commands;
mod model;
mod constants;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let command = args.get(1).map(String::as_str);
    
    match command {
        Some("init") => commands::init::run(),
        Some("add") => commands::add::run(&args),
        Some("diff") => commands::show_diff::run(),
        Some("write-tree") => commands::write_tree::run(),
        Some("read-tree") => commands::read_tree::run(&args),
        Some("commit-tree")=> commands::commit_tree::run(&args),
        _ => {
            eprintln!("usage: depot <command>");
            eprintln!("commands: init, add, diff, write-tree, read-tree, commit-tree");
            std::process::exit(1);
        }

    }
}
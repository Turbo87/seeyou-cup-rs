use seeyou_cup::CupFile;
use std::env;
use std::fs;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        eprintln!("Usage: {} <folder>", args[0]);
        process::exit(1);
    }

    let folder_path = &args[1];

    let entries = match fs::read_dir(folder_path) {
        Ok(entries) => entries,
        Err(e) => {
            eprintln!("Error reading folder: {}", e);
            process::exit(1);
        }
    };

    let mut cup_files = Vec::new();
    for entry in entries {
        let entry = match entry {
            Ok(entry) => entry,
            Err(e) => {
                eprintln!("Error reading entry: {}", e);
                continue;
            }
        };

        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("cup") {
            cup_files.push(path);
        }
    }

    if cup_files.is_empty() {
        println!("No .cup files found in {}", folder_path);
        return;
    }

    cup_files.sort();

    println!("Found {} .cup file(s)\n", cup_files.len());

    let mut success_count = 0;
    let mut error_count = 0;

    for path in &cup_files {
        let filename = path.file_name().and_then(|s| s.to_str()).unwrap_or("?");
        print!("Parsing {}... ", filename);

        match CupFile::from_path(path) {
            Ok(cup_file) => {
                println!(
                    "✓ ({} waypoints, {} tasks)",
                    cup_file.waypoints.len(),
                    cup_file.tasks.len()
                );
                success_count += 1;
            }
            Err(e) => {
                println!("✗");
                println!("  Error: {}", e);
                error_count += 1;
            }
        }
    }

    println!("\n{} successful, {} failed", success_count, error_count);

    if error_count > 0 {
        process::exit(1);
    }
}

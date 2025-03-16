mod core;

use std::path::PathBuf;
use std::{env, fs, io, path::Path, process};
use num_cpus;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 3 {
        print_help();
        process::exit(1);
    }

    let command = args[1].as_str();
    let path = args[2].as_str();
    let threads = num_cpus::get() / 2;

    rayon::ThreadPoolBuilder::new()
        .num_threads(threads)
        .build_global()
        .unwrap();
    println!(
        "Using {} threads for compression/decompression",
        rayon::current_num_threads()
    );

    let result = match command {
        "compress" => {
            if Path::new(path).is_dir() {
                compress_directory(path)
            } else {
                compress_file(path)
            }
        }
        "decompress" => {
            if path.ends_with(".tz") {
                decompress_file(path)
            } else {
                println!("Only .tz files can be decompressed");
                process::exit(1);
            }
        }
        "help" | "--help" | "-h" => {
            print_help();
            Ok(())
        }
        _ => {
            println!("Unknown command: {}", command);
            print_help();
            process::exit(1);
        }
    };

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}

fn compress_file(file_path: &str) -> io::Result<()> {
    let tz_file = format!("{}.tz", file_path);
    let file_contents = fs::read(file_path)?;
    let compressed = core::compression::compress(file_contents);

    fs::write(&tz_file, compressed)?;
    println!("Successfully compressed to {}", tz_file);
    Ok(())
}

fn decompress_file(file_path: &str) -> io::Result<()> {
    let path = Path::new(file_path);

    let output_name = if file_path.ends_with(".tz") {
        path.file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string()
    } else {
        format!("{}.decompressed", file_path)
    };

    let output_path = path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join(&output_name);

    let compressed_data = fs::read(file_path)?;
    let decompressed = core::decompression::decompress_binary(compressed_data)?;

    if decompressed.starts_with(b"TZ_DIR_ARCHIVE:") {
        fs::create_dir_all(&output_path)?;

        let content = String::from_utf8_lossy(&decompressed[15..]);
        let entries: Vec<&str> = content.split('\n').collect();

        let mut i = 0;
        while i < entries.len() {
            let entry = entries[i].trim();
            if entry.is_empty() {
                i += 1;
                continue;
            }

            let parts: Vec<&str> = entry.split(':').collect();
            if parts.len() != 2 {
                i += 1;
                continue;
            }

            let rel_path = parts[0];
            let size: usize = parts[1].parse().unwrap_or(0);

            let target_path = output_path.join(rel_path);

            if size > 0 && i + 1 < entries.len() {
                i += 1;
                let content = entries[i].as_bytes();

                if let Some(parent) = target_path.parent() {
                    fs::create_dir_all(parent)?;
                }

                fs::write(&target_path, content)?;
            } else {
                fs::create_dir_all(&target_path)?;
            }

            i += 1;
        }

        println!(
            "Successfully decompressed directory to {}",
            output_path.display()
        );
    } else {
        fs::write(&output_path, decompressed)?;
        println!("Successfully decompressed to {}", output_path.display());
    }

    Ok(())
}

fn compress_directory(dir_path: &str) -> io::Result<()> {
    let path = Path::new(dir_path);
    let dir_name = path
        .file_name()
        .unwrap_or_else(|| Path::new(dir_path).as_os_str())
        .to_string_lossy();

    let tz_file = format!("{}.tz", dir_name);

    let mut archive_data = Vec::new();
    archive_data.extend_from_slice(b"TZ_DIR_ARCHIVE:\n");

    let mut entries = Vec::new();
    match collect_directory_entries(path, &mut entries) {
        Ok(_) => (),
        Err(e) => return Err(e),
    }

    for entry in &entries {
        if entry.is_dir() {
            let rel_path = get_relative_path(path, entry);
            archive_data.extend_from_slice(format!("{}:0\n", rel_path).as_bytes());
        }
    }

    for entry in &entries {
        if !entry.is_dir() {
            let rel_path = get_relative_path(path, entry);
            let content = fs::read(entry)?;

            archive_data.extend_from_slice(format!("{}:{}\n", rel_path, content.len()).as_bytes());
            archive_data.extend_from_slice(&content);
            archive_data.push(b'\n');
        }
    }

    let compressed = core::compression::compress(archive_data);

    fs::write(&tz_file, compressed)?;
    println!("Successfully compressed directory to {}", tz_file);
    Ok(())
}

fn get_relative_path(base: &Path, path: &Path) -> String {
    if let Ok(rel_path) = path.strip_prefix(base) {
        rel_path.to_string_lossy().to_string()
    } else {
        path.file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string()
    }
}

fn collect_directory_entries(dir: &Path, entries: &mut Vec<PathBuf>) -> io::Result<()> {
    entries.push(dir.to_path_buf());

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            collect_directory_entries(&path, entries)?;
        } else {
            entries.push(path);
        }
    }

    Ok(())
}

fn print_help() {
    println!("Usage: tz <command> <path>");
    println!("Commands:");
    println!("  compress   - Compress a file or directory");
    println!("  decompress - Decompress a .tz file");
    println!("Examples:");
    println!("  tz compress file.txt     - Creates file.txt.tz");
    println!("  tz compress directory/   - Creates directory.tz");
    println!("  tz decompress file.tz    - Extracts to file");
}

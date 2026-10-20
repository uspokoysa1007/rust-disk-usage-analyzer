use std::env;
use std::fs;
use std::path::{Path, PathBuf};

struct DirEntryInfo {
    path: PathBuf,
    is_dir: bool,
    size: u64,
    file_count: usize,
}

fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

fn calculate_dir_size(path: &Path) -> (u64, usize) {
    let mut total_size = 0u64;
    let mut file_count = 0usize;

    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            let p = entry.path();
            if let Ok(metadata) = fs::symlink_metadata(&p) {
                if metadata.is_dir() {
                    let (sub_size, sub_files) = calculate_dir_size(&p);
                    total_size += sub_size;
                    file_count += sub_files;
                } else {
                    total_size += metadata.len();
                    file_count += 1;
                }
            }
        }
    }
    (total_size, file_count)
}

fn render_bar(percentage: f64, width: usize) -> String {
    let filled = ((percentage / 100.0) * width as f64).round() as usize;
    let filled = filled.min(width);
    let empty = width - filled;
    format!("[{}{}]", "=".repeat(filled), " ".repeat(empty))
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let target_dir = if args.len() > 1 {
        PathBuf::from(&args[1])
    } else {
        env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
    };

    println!("================================================================");
    println!("             RUST DISK USAGE ANALYZER                           ");
    println!("================================================================");
    println!("Scanning target: {}\n", target_dir.display());

    let mut items: Vec<DirEntryInfo> = Vec::new();
    let mut grand_total = 0u64;

    if let Ok(entries) = fs::read_dir(&target_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if let Ok(meta) = fs::symlink_metadata(&path) {
                if meta.is_dir() {
                    let (size, files) = calculate_dir_size(&path);
                    items.push(DirEntryInfo {
                        path,
                        is_dir: true,
                        size,
                        file_count: files,
                    });
                    grand_total += size;
                } else {
                    let size = meta.len();
                    items.push(DirEntryInfo {
                        path,
                        is_dir: false,
                        size,
                        file_count: 1,
                    });
                    grand_total += size;
                }
            }
        }
    } else {
        eprintln!("[-] Error: Unable to read directory: {}", target_dir.display());
        return;
    }

    // Sort descending by size
    items.sort_by(|a, b| b.size.cmp(&a.size));

    println!("{:<32} {:<12} {:<10} {}", "Entry Name", "Size", "Usage %", "Visual Bar");
    println!("{}", "-".repeat(70));

    for item in &items {
        let name = item.path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("???");
        let display_name = if item.is_dir {
            format!("{}/", name)
        } else {
            name.to_string()
        };

        let pct = if grand_total > 0 {
            (item.size as f64 / grand_total as f64) * 100.0
        } else {
            0.0
        };

        let bar = render_bar(pct, 12);
        let truncated_name = if display_name.len() > 30 {
            format!("{}...", &display_name[..27])
        } else {
            display_name
        };

        println!("{:<32} {:<12} {:>6.1}%    {}", 
                 truncated_name, 
                 format_size(item.size), 
                 pct, 
                 bar);
    }

    println!("{}", "-".repeat(70));
    println!("Total Scanned Volume: {} across {} top-level entries", format_size(grand_total), items.len());
    println!("================================================================");
}

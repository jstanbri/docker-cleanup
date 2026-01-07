use std::process::Command;
use std::io::{self, Write};
use std::path::PathBuf;
use std::collections::HashMap;

mod filesystem;

#[derive(Debug)]
struct ImageInfo {
    id: String,
    repository: String,
    tag: String,
    size: String,
}

#[derive(Debug)]
struct ContainerInfo {
    id: String,
    name: String,
    image: String,
    status: String,
}

fn main() {
    println!("Docker Cleanup Tool\n");
    
    // Check Docker images
    match list_images() {
        Ok(images) => {
            println!("═══ Docker Images ═══");
            if images.is_empty() {
                println!("No images found.\n");
            } else {
                for (i, img) in images.iter().enumerate() {
                    println!("{}. {} ({}:{})", i + 1, img.id, img.repository, img.tag);
                    println!("   Size: {}\n", img.size);
                }
                
                // Offer to remove dangling images
                if let Ok(dangling) = count_dangling_images() {
                    if dangling > 0 {
                        println!("Found {} dangling image(s) (not tagged)", dangling);
                        if prompt_yes_no("Remove dangling images?") {
                            remove_dangling_images();
                        }
                    }
                }
            }
        }
        Err(e) => eprintln!("Error listing images: {}", e),
    }
    
    // Check Docker containers
    match list_containers() {
        Ok(containers) => {
            println!("\n═══ Docker Containers ═══");
            if containers.is_empty() {
                println!("No containers found.\n");
            } else {
                let mut stopped = Vec::new();
                for (i, c) in containers.iter().enumerate() {
                    println!("{}. {} ({})", i + 1, c.id, c.name);
                    println!("   Image: {} | Status: {}\n", c.image, c.status);
                    
                    if c.status.starts_with("Exited") || c.status.starts_with("Created") {
                        stopped.push(c);
                    }
                }
                
                // Offer to remove stopped containers
                if !stopped.is_empty() {
                    println!("Found {} stopped container(s)", stopped.len());
                    if prompt_yes_no("Remove stopped containers?") {
                        remove_stopped_containers();
                    }
                }
            }
        }
        Err(e) => eprintln!("Error listing containers: {}", e),
    }
    
    // Show disk usage
    println!("\n═══ Docker Disk Usage ═══");
    show_disk_usage();
    
    // Offer full cleanup
    println!("\n═══ Additional Cleanup Options ═══");
    if prompt_yes_no("Run full system prune (removes unused data)?") {
        system_prune();
    }
    
    // Add filesystem cleanup
    println!("\n═══ Filesystem Cleanup ═══");
    if prompt_yes_no("Analyze filesystem for cleanup opportunities?") {
        filesystem_cleanup();
    }
}

fn list_images() -> Result<Vec<ImageInfo>, String> {
    let output = Command::new("docker")
        .args(&["images", "--format", "{{.ID}}|{{.Repository}}|{{.Tag}}|{{.Size}}"])
        .output()
        .map_err(|e| format!("Failed to execute docker command: {}", e))?;
    
    if !output.status.success() {
        return Err("Docker command failed".to_string());
    }
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    let images = stdout
        .lines()
        .filter_map(|line| {
            let parts: Vec<&str> = line.split('|').collect();
            if parts.len() == 4 {
                Some(ImageInfo {
                    id: parts[0].to_string(),
                    repository: parts[1].to_string(),
                    tag: parts[2].to_string(),
                    size: parts[3].to_string(),
                })
            } else {
                None
            }
        })
        .collect();
    
    Ok(images)
}

fn list_containers() -> Result<Vec<ContainerInfo>, String> {
    let output = Command::new("docker")
        .args(&["ps", "-a", "--format", "{{.ID}}|{{.Names}}|{{.Image}}|{{.Status}}"])
        .output()
        .map_err(|e| format!("Failed to execute docker command: {}", e))?;
    
    if !output.status.success() {
        return Err("Docker command failed".to_string());
    }
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    let containers = stdout
        .lines()
        .filter_map(|line| {
            let parts: Vec<&str> = line.split('|').collect();
            if parts.len() == 4 {
                Some(ContainerInfo {
                    id: parts[0].to_string(),
                    name: parts[1].to_string(),
                    image: parts[2].to_string(),
                    status: parts[3].to_string(),
                })
            } else {
                None
            }
        })
        .collect();
    
    Ok(containers)
}

fn count_dangling_images() -> Result<usize, String> {
    let output = Command::new("docker")
        .args(&["images", "-f", "dangling=true", "-q"])
        .output()
        .map_err(|e| format!("Failed to execute docker command: {}", e))?;
    
    Ok(String::from_utf8_lossy(&output.stdout).lines().count())
}

fn remove_dangling_images() {
    println!("Removing dangling images...");
    let output = Command::new("docker")
        .args(&["image", "prune", "-f"])
        .output();
    
    match output {
        Ok(o) => {
            println!("{}", String::from_utf8_lossy(&o.stdout));
        }
        Err(e) => eprintln!("Error: {}", e),
    }
}

fn remove_stopped_containers() {
    println!("Removing stopped containers...");
    let output = Command::new("docker")
        .args(&["container", "prune", "-f"])
        .output();
    
    match output {
        Ok(o) => {
            println!("{}", String::from_utf8_lossy(&o.stdout));
        }
        Err(e) => eprintln!("Error: {}", e),
    }
}

fn show_disk_usage() {
    let output = Command::new("docker")
        .args(&["system", "df"])
        .output();
    
    match output {
        Ok(o) => {
            println!("{}", String::from_utf8_lossy(&o.stdout));
        }
        Err(e) => eprintln!("Error: {}", e),
    }
}

fn system_prune() {
    println!("Running system prune...");
    let output = Command::new("docker")
        .args(&["system", "prune", "-f"])
        .output();
    
    match output {
        Ok(o) => {
            println!("{}", String::from_utf8_lossy(&o.stdout));
        }
        Err(e) => eprintln!("Error: {}", e),
    }
}

fn prompt_yes_no(question: &str) -> bool {
    print!("{} (y/N): ", question);
    io::stdout().flush().unwrap();
    
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    
    matches!(input.trim().to_lowercase().as_str(), "y" | "yes")
}

fn prompt_for_directory(default_dir: &str) -> PathBuf {
    print!("Enter directory to analyze (default: {}): ", default_dir);
    io::stdout().flush().unwrap();
    
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    let input = input.trim();
    
    if input.is_empty() {
        PathBuf::from(default_dir)
    } else {
        PathBuf::from(input)
    }
}

fn format_size(size: u64) -> String {
    use humansize::{format_size, DECIMAL};
    format_size(size, DECIMAL)
}

fn filesystem_cleanup() {
    // Get home directory as default
    let home_dir = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    let default_path = home_dir.to_string_lossy().to_string();
    
    let scan_path = prompt_for_directory(&default_path);
    
    if !scan_path.exists() {
        eprintln!("Error: Directory does not exist: {}", scan_path.display());
        return;
    }
    
    println!("\nScanning {}...\n", scan_path.display());
    
    // Find large files
    println!("Scanning for large files...");
    let large_files = filesystem::find_large_files(
        &scan_path, 
        100, // 100MB minimum
        |_path, _count| {}
    );
    
    // Find duplicates
    println!("\nChecking for duplicates...");
    let duplicates = filesystem::find_duplicates(
        &scan_path,
        |_msg, _current, _total| {}
    );
    
    // Find cache directories
    println!("\nIdentifying cache directories...");
    let cache_dirs = filesystem::find_cache_directories(
        &scan_path,
        |_path| {}
    );
    
    // Display results
    println!("\n═══ Scan Results ═══\n");
    
    // Top 10 largest files
    println!("Top 10 Largest Files:");
    if large_files.is_empty() {
        println!("  No large files found (>100MB)\n");
    } else {
        for (i, file) in large_files.iter().take(10).enumerate() {
            println!("{}. {} - {}", 
                i + 1, 
                format_size(file.size), 
                file.path.display()
            );
        }
        println!();
    }
    
    // Duplicate files
    let mut total_duplicate_size = 0u64;
    
    for group in &duplicates {
        if group.len() > 1 {
            let file_size = group[0].size;
            total_duplicate_size += file_size * (group.len() - 1) as u64;
        }
    }
    
    println!("Duplicate Files ({} groups, {} reclaimable):", 
        duplicates.len(), 
        format_size(total_duplicate_size)
    );
    
    if duplicates.is_empty() {
        println!("  No duplicate files found\n");
    } else {
        for (i, group) in duplicates.iter().take(5).enumerate() {
            if i >= 5 { break; }
            println!("  Group {}: {} copies ({} each)", 
                i + 1, 
                group.len(), 
                format_size(group[0].size)
            );
            for file in group.iter().take(3) {
                println!("    - {}", file.path.display());
            }
            if group.len() > 3 {
                println!("    ... and {} more", group.len() - 3);
            }
        }
        if duplicates.len() > 5 {
            println!("  ... and {} more groups", duplicates.len() - 5);
        }
        println!();
    }
    
    // Cache directories
    let mut cache_by_type: HashMap<String, Vec<&filesystem::CacheInfo>> = HashMap::new();
    let mut total_cache_size = 0u64;
    
    for cache in &cache_dirs {
        cache_by_type.entry(cache.cache_type.clone()).or_insert_with(Vec::new).push(cache);
        total_cache_size += cache.size;
    }
    
    println!("Cache Directories ({} total):", format_size(total_cache_size));
    if cache_dirs.is_empty() {
        println!("  No cache directories found\n");
    } else {
        for (cache_type, caches) in cache_by_type.iter() {
            let type_size: u64 = caches.iter().map(|c| c.size).sum();
            println!("  {}: {} ({} locations)", 
                cache_type, 
                format_size(type_size), 
                caches.len()
            );
        }
        println!();
    }
    
    // Offer cleanup options
    if !duplicates.is_empty() {
        println!("═══ Cleanup Options ═══");
        println!("Note: Duplicate and cache cleanup requires manual review.");
        println!("Please review the paths above and delete files manually if desired.\n");
    }
    
    println!("Total potential savings: {}", 
        format_size(total_duplicate_size + total_cache_size)
    );
}

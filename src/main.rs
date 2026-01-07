use std::process::Command;
use std::io::{self, Write};
use std::path::Path;

mod filesystem;
use filesystem::{AnalysisConfig, analyze_disk, format_file_size, delete_file, delete_directory};

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
    
    // Filesystem cleanup section
    println!("\n═══ Filesystem Cleanup ═══");
    if prompt_yes_no("Run filesystem analysis?") {
        run_filesystem_cleanup();
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

fn run_filesystem_cleanup() {
    // Determine scan path
    print!("Enter directory to scan (or press Enter for current directory): ");
    io::stdout().flush().unwrap();
    
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    let input = input.trim();
    
    let scan_path = if input.is_empty() {
        Path::new(".").to_path_buf()
    } else {
        Path::new(input).to_path_buf()
    };
    
    if !scan_path.exists() {
        eprintln!("Error: Path does not exist");
        return;
    }
    
    println!("\n═══ Filesystem Analysis ═══");
    println!("Scanning: {}\n", scan_path.display());
    
    let config = AnalysisConfig::default();
    let analysis = analyze_disk(&scan_path, config.clone());
    
    // Display large files
    println!("\n═══ Top {} Largest Files ═══", config.max_large_files);
    if analysis.large_files.is_empty() {
        println!("No large files found (threshold: {} MB)", config.min_file_size_mb);
    } else {
        for (i, file) in analysis.large_files.iter().take(config.max_large_files).enumerate() {
            println!("{}. {} ({})", 
                i + 1, 
                file.path.display(), 
                format_file_size(file.size)
            );
        }
    }
    
    // Display duplicate files
    println!("\n═══ Duplicate Files ═══");
    if analysis.duplicate_groups.is_empty() {
        println!("No duplicate files found");
    } else {
        let duplicate_reclaimable: u64 = analysis.duplicate_groups
            .iter()
            .map(|group| {
                if group.len() > 1 {
                    group[0].size * (group.len() as u64 - 1)
                } else {
                    0
                }
            })
            .sum();
        
        println!("Found {} groups, {} reclaimable:", 
            analysis.duplicate_groups.len(),
            format_file_size(duplicate_reclaimable)
        );
        
        for (i, group) in analysis.duplicate_groups.iter().take(5).enumerate() {
            if let Some(first) = group.first() {
                println!("\nGroup {}: {} ({} copies, {} each):",
                    i + 1,
                    first.path.file_name().unwrap_or_default().to_string_lossy(),
                    group.len(),
                    format_file_size(first.size)
                );
                for file in group {
                    println!("  - {}", file.path.display());
                }
            }
        }
        
        if analysis.duplicate_groups.len() > 5 {
            println!("\n... and {} more groups", analysis.duplicate_groups.len() - 5);
        }
    }
    
    // Display cache directories
    println!("\n═══ Cache Directories ═══");
    if analysis.cache_dirs.is_empty() {
        println!("No cache directories found");
    } else {
        let cache_total: u64 = analysis.cache_dirs.iter().map(|c| c.size).sum();
        println!("Total: {}\n", format_file_size(cache_total));
        
        let mut cache_by_type: std::collections::HashMap<String, Vec<&filesystem::CacheInfo>> = 
            std::collections::HashMap::new();
        
        for cache in &analysis.cache_dirs {
            cache_by_type.entry(cache.cache_type.clone())
                .or_insert_with(Vec::new)
                .push(cache);
        }
        
        for (cache_type, caches) in cache_by_type.iter() {
            let type_total: u64 = caches.iter().map(|c| c.size).sum();
            println!("{}: {} - {} directories", 
                cache_type, 
                format_file_size(type_total),
                caches.len()
            );
        }
    }
    
    // Display old files
    println!("\n═══ Old Files ═══");
    if analysis.old_files.is_empty() {
        println!("No old files found (threshold: {} days)", config.old_file_days);
    } else {
        let old_files_size: u64 = analysis.old_files.iter().map(|f| f.size).sum();
        println!("Files not accessed in {}+ days: {} files, {}",
            config.old_file_days,
            analysis.old_files.len(),
            format_file_size(old_files_size)
        );
    }
    
    // Display total
    println!("\n═══ Summary ═══");
    println!("Total Reclaimable Space: ~{}", format_file_size(analysis.total_reclaimable));
    
    // Offer cleanup options
    println!("\n═══ Cleanup Options ═══");
    
    // Option 1: Remove duplicate files
    if !analysis.duplicate_groups.is_empty() {
        if prompt_yes_no("Remove duplicate files (keep one copy)?") {
            let mut removed_count = 0;
            let mut removed_size = 0u64;
            
            for group in &analysis.duplicate_groups {
                if group.len() > 1 {
                    // Keep the first file, remove the rest
                    for file in group.iter().skip(1) {
                        match delete_file(&file.path) {
                            Ok(_) => {
                                println!("Removed: {}", file.path.display());
                                removed_count += 1;
                                removed_size += file.size;
                            }
                            Err(e) => {
                                eprintln!("Error removing {}: {}", file.path.display(), e);
                            }
                        }
                    }
                }
            }
            
            println!("Removed {} duplicate files, recovered {}", 
                removed_count, 
                format_file_size(removed_size)
            );
        }
    }
    
    // Option 2: Clear cache directories
    if !analysis.cache_dirs.is_empty() {
        if prompt_yes_no("Clear cache directories?") {
            println!("Select cache types to clear:");
            
            let mut cache_by_type: std::collections::HashMap<String, Vec<&filesystem::CacheInfo>> = 
                std::collections::HashMap::new();
            
            for cache in &analysis.cache_dirs {
                cache_by_type.entry(cache.cache_type.clone())
                    .or_insert_with(Vec::new)
                    .push(cache);
            }
            
            for (i, (cache_type, caches)) in cache_by_type.iter().enumerate() {
                let type_total: u64 = caches.iter().map(|c| c.size).sum();
                println!("{}. {} ({}, {} directories)", 
                    i + 1,
                    cache_type, 
                    format_file_size(type_total),
                    caches.len()
                );
            }
            
            print!("Enter numbers to clear (comma-separated, or 'all'): ");
            io::stdout().flush().unwrap();
            
            let mut input = String::new();
            io::stdin().read_line(&mut input).unwrap();
            let input = input.trim();
            
            if !input.is_empty() {
                let mut removed_size = 0u64;
                let cache_types: Vec<_> = cache_by_type.keys().cloned().collect();
                
                if input.to_lowercase() == "all" {
                    for cache in &analysis.cache_dirs {
                        match delete_directory(&cache.path) {
                            Ok(_) => {
                                println!("Removed: {}", cache.path.display());
                                removed_size += cache.size;
                            }
                            Err(e) => {
                                eprintln!("Error removing {}: {}", cache.path.display(), e);
                            }
                        }
                    }
                } else {
                    let selections: Vec<usize> = input
                        .split(',')
                        .filter_map(|s| s.trim().parse::<usize>().ok())
                        .collect();
                    
                    for idx in selections {
                        if idx > 0 && idx <= cache_types.len() {
                            let cache_type = &cache_types[idx - 1];
                            if let Some(caches) = cache_by_type.get(cache_type) {
                                for cache in caches {
                                    match delete_directory(&cache.path) {
                                        Ok(_) => {
                                            println!("Removed: {}", cache.path.display());
                                            removed_size += cache.size;
                                        }
                                        Err(e) => {
                                            eprintln!("Error removing {}: {}", cache.path.display(), e);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                
                println!("Recovered {}", format_file_size(removed_size));
            }
        }
    }
    
    // Option 3: Remove old files
    if !analysis.old_files.is_empty() {
        if prompt_yes_no(&format!("Remove files not accessed in {}+ days?", config.old_file_days)) {
            print!("Are you sure? This will delete {} files (y/N): ", analysis.old_files.len());
            io::stdout().flush().unwrap();
            
            let mut input = String::new();
            io::stdin().read_line(&mut input).unwrap();
            
            if matches!(input.trim().to_lowercase().as_str(), "y" | "yes") {
                let mut removed_count = 0;
                let mut removed_size = 0u64;
                
                for file in &analysis.old_files {
                    match delete_file(&file.path) {
                        Ok(_) => {
                            removed_count += 1;
                            removed_size += file.size;
                        }
                        Err(e) => {
                            eprintln!("Error removing {}: {}", file.path.display(), e);
                        }
                    }
                }
                
                println!("Removed {} old files, recovered {}", 
                    removed_count, 
                    format_file_size(removed_size)
                );
            }
        }
    }
    
    println!("\nFilesystem cleanup complete!");
}

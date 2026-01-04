use std::process::Command;
use std::io::{self, Write};

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

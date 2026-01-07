use std::collections::HashMap;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use sha2::{Sha256, Digest};
use indicatif::{ProgressBar, ProgressStyle};

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct FileInfo {
    pub path: PathBuf,
    pub size: u64,
    pub last_accessed: Option<std::time::SystemTime>,
    pub last_modified: Option<std::time::SystemTime>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct CacheInfo {
    pub path: PathBuf,
    pub cache_type: String,
    pub size: u64,
}

/// Find files larger than min_size_mb in the given directory
pub fn find_large_files<F>(root: &Path, min_size_mb: u64, mut progress_callback: F) -> Vec<FileInfo>
where
    F: FnMut(&Path, usize),
{
    let min_size_bytes = min_size_mb * 1024 * 1024;
    let mut large_files = Vec::new();
    let mut file_count = 0;
    
    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .unwrap()
    );
    
    for entry in WalkDir::new(root)
        .follow_links(false)
        .into_iter()
        .filter_entry(|e| !is_system_directory(e.path()))
    {
        match entry {
            Ok(entry) => {
                let path = entry.path();
                file_count += 1;
                
                if file_count % 100 == 0 {
                    spinner.set_message(format!("{} files scanned | Current: {}", 
                        file_count, 
                        path.display()
                    ));
                    progress_callback(path, file_count);
                }
                
                if entry.file_type().is_file() {
                    if let Ok(metadata) = entry.metadata() {
                        let size = metadata.len();
                        if size >= min_size_bytes {
                            large_files.push(FileInfo {
                                path: path.to_path_buf(),
                                size,
                                last_accessed: metadata.accessed().ok(),
                                last_modified: metadata.modified().ok(),
                            });
                        }
                    }
                }
            }
            Err(e) => {
                // Gracefully handle permission errors
                if e.io_error().map(|io| io.kind()) == Some(std::io::ErrorKind::PermissionDenied) {
                    eprintln!("Warning: Permission denied: {}", e);
                }
            }
        }
    }
    
    spinner.finish_with_message(format!("✓ Scanned {} files", file_count));
    
    // Sort by size, largest first
    large_files.sort_by(|a, b| b.size.cmp(&a.size));
    large_files
}

/// Find duplicate files using SHA-256 hashing
pub fn find_duplicates<F>(root: &Path, mut progress_callback: F) -> Vec<Vec<FileInfo>>
where
    F: FnMut(&str, usize, usize),
{
    // First pass: group files by size
    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .unwrap()
    );
    spinner.set_message("Scanning files by size...");
    
    let mut size_groups: HashMap<u64, Vec<FileInfo>> = HashMap::new();
    let mut file_count = 0;
    
    for entry in WalkDir::new(root)
        .follow_links(false)
        .into_iter()
        .filter_entry(|e| !is_system_directory(e.path()))
    {
        if let Ok(entry) = entry {
            if entry.file_type().is_file() {
                if let Ok(metadata) = entry.metadata() {
                    file_count += 1;
                    let size = metadata.len();
                    
                    // Only consider files larger than 1KB to avoid noise
                    if size > 1024 {
                        let file_info = FileInfo {
                            path: entry.path().to_path_buf(),
                            size,
                            last_accessed: metadata.accessed().ok(),
                            last_modified: metadata.modified().ok(),
                        };
                        size_groups.entry(size).or_insert_with(Vec::new).push(file_info);
                    }
                }
            }
        }
    }
    
    spinner.finish_with_message(format!("✓ Found {} files", file_count));
    
    // Second pass: hash files that have matching sizes
    let candidates: Vec<_> = size_groups
        .into_iter()
        .filter(|(_, files)| files.len() > 1)
        .flat_map(|(_, files)| files)
        .collect();
    
    let total_to_hash = candidates.len();
    if total_to_hash == 0 {
        return Vec::new();
    }
    
    let pb = ProgressBar::new(total_to_hash as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("[{bar:40.cyan/blue}] {pos}/{len} files hashed ({percent}%)")
            .unwrap()
            .progress_chars("█▓▒░ ")
    );
    
    let mut hash_groups: HashMap<String, Vec<FileInfo>> = HashMap::new();
    
    for (idx, file_info) in candidates.into_iter().enumerate() {
        pb.set_position(idx as u64);
        progress_callback("Hashing files for duplicates...", idx, total_to_hash);
        
        if let Ok(hash) = compute_file_hash(&file_info.path) {
            hash_groups.entry(hash).or_insert_with(Vec::new).push(file_info);
        }
    }
    
    pb.finish_with_message("✓ Hashing complete");
    
    // Return only groups with duplicates
    hash_groups
        .into_iter()
        .filter(|(_, files)| files.len() > 1)
        .map(|(_, files)| files)
        .collect()
}

/// Find common cache directories
pub fn find_cache_directories<F>(root: &Path, mut progress_callback: F) -> Vec<CacheInfo>
where
    F: FnMut(&Path),
{
    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .unwrap()
    );
    
    let cache_patterns = vec![
        ("node_modules", "npm/yarn"),
        ("target", "Rust/Cargo"),
        ("__pycache__", "Python"),
        (".cache", "Generic cache"),
        ("build", "Build output"),
        (".pytest_cache", "pytest"),
        (".mypy_cache", "mypy"),
        ("dist", "Distribution"),
    ];
    
    let mut cache_dirs = Vec::new();
    let mut scanned_count = 0;
    
    for entry in WalkDir::new(root)
        .follow_links(false)
        .max_depth(10) // Limit depth to avoid going too deep
        .into_iter()
        .filter_entry(|e| !is_system_directory(e.path()))
    {
        if let Ok(entry) = entry {
            scanned_count += 1;
            
            if scanned_count % 50 == 0 {
                spinner.set_message(format!("Scanning... {}", entry.path().display()));
                progress_callback(entry.path());
            }
            
            if entry.file_type().is_dir() {
                let dir_name = entry.file_name().to_string_lossy();
                
                for (pattern, cache_type) in &cache_patterns {
                    if dir_name == *pattern {
                        if let Ok(size) = calculate_dir_size(entry.path()) {
                            cache_dirs.push(CacheInfo {
                                path: entry.path().to_path_buf(),
                                cache_type: cache_type.to_string(),
                                size,
                            });
                        }
                    }
                }
            }
        }
    }
    
    spinner.finish_with_message("✓ Cache scan complete");
    cache_dirs
}

/// Check if a path is a system directory that should be skipped
fn is_system_directory(path: &Path) -> bool {
    let path_str = path.to_string_lossy();
    
    // Skip these exact paths or direct children (but not descendants)
    let skip_patterns = vec![
        "/.git/",
        "/proc/",
        "/sys/",
        "/dev/",
        "/run/",
        "/.Trash/",
        "/Library/Caches/",
        "/System/",
        "/Volumes/",
    ];
    
    // Check if the path contains any of the skip patterns
    // This will skip e.g., /proc/... but not /tmp/proc/
    for pattern in &skip_patterns {
        if path_str.contains(pattern) {
            return true;
        }
    }
    
    // Special handling for .git directories at any level
    if let Some(file_name) = path.file_name() {
        if file_name == ".git" {
            return true;
        }
    }
    
    false
}

/// Compute SHA-256 hash of a file
fn compute_file_hash(path: &Path) -> std::io::Result<String> {
    let mut file = fs::File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = vec![0; 8192]; // 8KB buffer for reading
    
    loop {
        let bytes_read = file.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }
    
    Ok(format!("{:x}", hasher.finalize()))
}

/// Calculate the total size of a directory
fn calculate_dir_size(path: &Path) -> std::io::Result<u64> {
    let mut total_size = 0;
    
    for entry in WalkDir::new(path).follow_links(false) {
        if let Ok(entry) = entry {
            if entry.file_type().is_file() {
                if let Ok(metadata) = entry.metadata() {
                    total_size += metadata.len();
                }
            }
        }
    }
    
    Ok(total_size)
}

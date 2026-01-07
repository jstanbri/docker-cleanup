use std::path::{Path, PathBuf};
use std::fs::{self, File};
use std::io::{self, Read};
use std::collections::{HashMap, HashSet};
use std::time::{SystemTime, Duration};
use walkdir::WalkDir;
use sha2::{Sha256, Digest};
use humansize::{format_size, BINARY};

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct FileInfo {
    pub path: PathBuf,
    pub size: u64,
    pub last_accessed: SystemTime,
    pub last_modified: SystemTime,
}

#[derive(Debug, Clone)]
pub struct CacheInfo {
    pub path: PathBuf,
    pub cache_type: String,
    pub size: u64,
}

#[derive(Debug)]
pub struct DiskAnalysis {
    pub large_files: Vec<FileInfo>,
    pub duplicate_groups: Vec<Vec<FileInfo>>,
    pub old_files: Vec<FileInfo>,
    pub cache_dirs: Vec<CacheInfo>,
    pub total_reclaimable: u64,
}

#[derive(Debug, Clone)]
pub struct AnalysisConfig {
    pub min_file_size_mb: u64,
    pub old_file_days: u64,
    pub max_large_files: usize,
}

impl Default for AnalysisConfig {
    fn default() -> Self {
        Self {
            min_file_size_mb: 100,
            old_file_days: 180,
            max_large_files: 10,
        }
    }
}

/// Check if a path should be excluded from scanning
fn should_exclude_path(path: &Path) -> bool {
    let excluded_dirs = [
        "/proc", "/sys", "/dev", "/run", "/boot",
        "System Volume Information", "Windows", "Program Files",
        "AppData/Local/Temp", "$Recycle.Bin",
    ];
    
    let path_str = path.to_string_lossy();
    
    // Check for exact matches at the start of path for system directories
    for excluded in &excluded_dirs {
        if path_str.starts_with(excluded) || path_str.contains(&format!("/{}/", excluded)) {
            return true;
        }
    }
    false
}

/// Find large files above a certain size threshold
pub fn find_large_files(root: &Path, min_size_mb: u64) -> Vec<FileInfo> {
    let min_size_bytes = min_size_mb * 1024 * 1024;
    let mut files = Vec::new();
    
    for entry in WalkDir::new(root)
        .follow_links(false)
        .into_iter()
        .filter_entry(|e| !should_exclude_path(e.path()))
    {
        match entry {
            Ok(entry) => {
                if entry.file_type().is_file() {
                    if let Ok(metadata) = entry.metadata() {
                        let size = metadata.len();
                        if size >= min_size_bytes {
                            if let (Ok(accessed), Ok(modified)) = (
                                metadata.accessed(),
                                metadata.modified(),
                            ) {
                                files.push(FileInfo {
                                    path: entry.path().to_path_buf(),
                                    size,
                                    last_accessed: accessed,
                                    last_modified: modified,
                                });
                            }
                        }
                    }
                }
            }
            Err(_) => continue, // Skip permission errors
        }
    }
    
    // Sort by size descending
    files.sort_by(|a, b| b.size.cmp(&a.size));
    files
}

/// Calculate SHA-256 hash of a file
fn hash_file(path: &Path) -> io::Result<String> {
    let mut file = File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 8192];
    
    loop {
        let bytes_read = file.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }
    
    Ok(format!("{:x}", hasher.finalize()))
}

/// Find duplicate files using SHA-256 content hashing
pub fn find_duplicates(root: &Path) -> Vec<Vec<FileInfo>> {
    let mut size_groups: HashMap<u64, Vec<FileInfo>> = HashMap::new();
    
    // First pass: group files by size
    for entry in WalkDir::new(root)
        .follow_links(false)
        .into_iter()
        .filter_entry(|e| !should_exclude_path(e.path()))
    {
        if let Ok(entry) = entry {
            if entry.file_type().is_file() {
                if let Ok(metadata) = entry.metadata() {
                    let size = metadata.len();
                    // Only consider files larger than 1KB to avoid noise
                    if size > 1024 {
                        if let (Ok(accessed), Ok(modified)) = (
                            metadata.accessed(),
                            metadata.modified(),
                        ) {
                            let file_info = FileInfo {
                                path: entry.path().to_path_buf(),
                                size,
                                last_accessed: accessed,
                                last_modified: modified,
                            };
                            size_groups.entry(size).or_insert_with(Vec::new).push(file_info);
                        }
                    }
                }
            }
        }
    }
    
    // Second pass: hash files with same size
    let mut hash_groups: HashMap<String, Vec<FileInfo>> = HashMap::new();
    
    for (_, files) in size_groups {
        if files.len() > 1 {
            for file in files {
                if let Ok(hash) = hash_file(&file.path) {
                    hash_groups.entry(hash).or_insert_with(Vec::new).push(file);
                }
            }
        }
    }
    
    // Return only groups with duplicates
    hash_groups
        .into_iter()
        .filter(|(_, files)| files.len() > 1)
        .map(|(_, files)| files)
        .collect()
}

/// Find files not accessed in X days
pub fn find_old_files(root: &Path, days: u64) -> Vec<FileInfo> {
    let threshold = Duration::from_secs(days * 24 * 60 * 60);
    let now = SystemTime::now();
    let mut old_files = Vec::new();
    
    for entry in WalkDir::new(root)
        .follow_links(false)
        .into_iter()
        .filter_entry(|e| !should_exclude_path(e.path()))
    {
        if let Ok(entry) = entry {
            if entry.file_type().is_file() {
                if let Ok(metadata) = entry.metadata() {
                    if let Ok(accessed) = metadata.accessed() {
                        if let Ok(duration_since) = now.duration_since(accessed) {
                            if duration_since >= threshold {
                                if let Ok(modified) = metadata.modified() {
                                    old_files.push(FileInfo {
                                        path: entry.path().to_path_buf(),
                                        size: metadata.len(),
                                        last_accessed: accessed,
                                        last_modified: modified,
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    
    old_files
}

/// Calculate total size of a directory
fn calculate_dir_size(path: &Path) -> u64 {
    WalkDir::new(path)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter_map(|e| e.metadata().ok())
        .filter(|m| m.is_file())
        .map(|m| m.len())
        .sum()
}

/// Find common cache and build directories
pub fn find_cache_directories(root: &Path) -> Vec<CacheInfo> {
    let cache_patterns = [
        ("node_modules", "npm/yarn"),
        ("target", "Rust/Cargo"),
        (".cache", "System cache"),
        ("__pycache__", "Python"),
        (".npm", "npm cache"),
        (".cargo/registry", "Cargo registry"),
        (".pip", "pip cache"),
        ("dist", "Build output"),
        ("build", "Build output"),
        (".gradle", "Gradle cache"),
        (".m2/repository", "Maven cache"),
    ];
    
    let mut cache_dirs = Vec::new();
    let mut seen_paths = HashSet::new();
    
    for entry in WalkDir::new(root)
        .follow_links(false)
        .max_depth(6) // Limit depth for performance
        .into_iter()
        .filter_entry(|e| !should_exclude_path(e.path()))
    {
        if let Ok(entry) = entry {
            if entry.file_type().is_dir() {
                let path = entry.path();
                let path_str = path.to_string_lossy();
                
                for (pattern, cache_type) in &cache_patterns {
                    if path_str.ends_with(pattern) && !seen_paths.contains(path) {
                        let size = calculate_dir_size(path);
                        if size > 0 {
                            cache_dirs.push(CacheInfo {
                                path: path.to_path_buf(),
                                cache_type: cache_type.to_string(),
                                size,
                            });
                            seen_paths.insert(path.to_path_buf());
                        }
                    }
                }
            }
        }
    }
    
    // Sort by size descending
    cache_dirs.sort_by(|a, b| b.size.cmp(&a.size));
    cache_dirs
}

/// Main entry point for disk analysis
pub fn analyze_disk(root: &Path, config: AnalysisConfig) -> DiskAnalysis {
    println!("Scanning for large files...");
    let large_files = find_large_files(root, config.min_file_size_mb);
    
    println!("Scanning for duplicate files...");
    let duplicate_groups = find_duplicates(root);
    
    println!("Scanning for old files...");
    let old_files = find_old_files(root, config.old_file_days);
    
    println!("Scanning for cache directories...");
    let cache_dirs = find_cache_directories(root);
    
    // Calculate total reclaimable space
    let duplicate_reclaimable: u64 = duplicate_groups
        .iter()
        .map(|group| {
            // Can reclaim n-1 copies
            if group.len() > 1 {
                group[0].size * (group.len() as u64 - 1)
            } else {
                0
            }
        })
        .sum();
    
    let cache_reclaimable: u64 = cache_dirs.iter().map(|c| c.size).sum();
    let old_files_reclaimable: u64 = old_files.iter().map(|f| f.size).sum();
    
    let total_reclaimable = duplicate_reclaimable + cache_reclaimable + old_files_reclaimable;
    
    DiskAnalysis {
        large_files,
        duplicate_groups,
        old_files,
        cache_dirs,
        total_reclaimable,
    }
}

/// Format file size in human-readable format
pub fn format_file_size(size: u64) -> String {
    format_size(size, BINARY)
}

/// Delete a file safely
pub fn delete_file(path: &Path) -> io::Result<()> {
    fs::remove_file(path)
}

/// Delete a directory recursively
pub fn delete_directory(path: &Path) -> io::Result<()> {
    fs::remove_dir_all(path)
}

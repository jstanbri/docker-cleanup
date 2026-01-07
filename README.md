# docker-cleanup

A utility to clean up Docker environments and analyze your filesystem to recover disk space.

## Features

### Docker Cleanup
- List and analyze Docker images and containers
- Identify and remove dangling images
- Remove stopped containers
- Show Docker disk usage
- Run full system prune to reclaim space

### Filesystem Analysis
- **Find Large Files**: Scan directories for files above 100MB
- **Detect Duplicates**: Identify duplicate files using SHA-256 content hashing
- **Identify Cache Directories**: Find common cache/build directories:
  - `node_modules/` (npm/yarn)
  - `target/` (Rust/Cargo)
  - `__pycache__/` (Python)
  - `.cache/` directories
  - And more...

All filesystem operations include real-time progress feedback with professional progress bars and spinners.

## Installation

```bash
cargo build --release
```

The binary will be available at `target/release/docker-cleanup`.

## Usage

Simply run the tool and follow the interactive prompts:

```bash
./docker-cleanup
```

The tool will:
1. Analyze your Docker environment
2. Prompt you to clean up Docker resources
3. Optionally analyze your filesystem for cleanup opportunities
4. Display detailed reports with human-readable sizes
5. Ask for confirmation before any deletion

## Safety Features

- **Always asks for confirmation** before deleting anything
- **Shows full paths** before any deletion
- **Handles permission errors gracefully**
- **Skips system directories** by default (.git, /proc, /sys, etc.)
- **Manual review required** for duplicate and cache file cleanup

## Example Output

```
═══ Filesystem Cleanup ═══
Analyze filesystem for cleanup opportunities? (y/N): y
Enter directory to analyze (default: /home/user): 

Scanning /home/user...

Scanning for large files...
✓ Scanned 1,234 files

Checking for duplicates...
[████████████████████████████████████████] 100%

Identifying cache directories...
✓ Cache scan complete

═══ Scan Results ═══

Top 10 Largest Files:
1. 157.29 MB - /home/user/downloads/video.mp4
2. 125.83 MB - /home/user/downloads/ubuntu.iso

Duplicate Files (1 groups, 20.48 kB reclaimable):
  Group 1: 3 copies (10.24 kB each)
    - /home/user/documents/document.pdf
    - /home/user/downloads/document.pdf
    - /home/user/backup/document.pdf

Cache Directories (2.5 GB total):
  npm/yarn: 1.2 GB (8 locations)
  Rust/Cargo: 800 MB (3 locations)
  Python: 500 MB (12 locations)

Total potential savings: 2.52 GB
```

## Dependencies

- `walkdir` - Efficient filesystem traversal
- `sha2` - SHA-256 hashing for duplicate detection
- `indicatif` - Professional progress bars and spinners
- `humansize` - Human-readable file sizes
- `dirs` - Cross-platform home directory detection

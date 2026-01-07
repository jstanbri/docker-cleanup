# docker-cleanup
A utility to clean up Docker environments and analyze filesystem disk space usage

## Features

### Docker Cleanup
- List and remove Docker images
- Remove dangling (untagged) images
- List and remove stopped containers
- Show Docker disk usage
- Run full system prune to remove unused data

### Filesystem Analysis and Cleanup
- **Large Files Detection**: Find files above a configurable size threshold (default 100MB)
- **Duplicate Detection**: Identify duplicate files using SHA-256 content hashing
- **Cache Directory Discovery**: Locate common cache/build directories:
  - `node_modules/` (npm/yarn)
  - `target/` (Rust/Cargo)
  - `.cache/` (various)
  - `__pycache__/` (Python)
  - `.npm/`, `.cargo/registry/`, `.pip/`
  - And more...
- **Old Files**: Find files not accessed in X days (default 180 days)
- **Interactive Cleanup**: Safe deletion with user confirmation
- **Space Recovery Report**: Shows total reclaimable space

## Installation

```bash
cargo build --release
```

## Usage

Run the utility:
```bash
cargo run
```

Or run the compiled binary:
```bash
./target/release/docker-cleanup
```

The tool will:
1. First analyze and offer to clean up Docker resources
2. Then offer to analyze the filesystem for disk space opportunities

When running filesystem analysis, you can:
- Scan a specific directory or the current directory
- View a comprehensive report of:
  - Top 10 largest files
  - Groups of duplicate files
  - Cache directories by type
  - Old files summary
  - Total reclaimable space estimate
- Choose which items to clean up interactively

## Safety Features

- All deletions require user confirmation
- System directories are excluded by default
- Full file paths shown before deletion
- Graceful handling of permission errors
- Preserves one copy when removing duplicates

## Example Output

```
═══ Filesystem Analysis ═══
Scanning: /home/user/projects

Top 10 Largest Files:
1. /home/user/videos/movie.mp4 (150 MiB)

Duplicate Files (1 groups, 20 KiB reclaimable):
Group 1: document.pdf (3 copies, 10 KiB each):
  - /home/user/downloads/document.pdf
  - /home/user/documents/document.pdf
  - /home/user/backup/document.pdf

Cache Directories (8 MiB total):
npm/yarn: 5 MiB - 1 directories

Old Files (not accessed in 180+ days):
No old files found

Total Reclaimable Space: ~8.02 MiB
```

## License

See LICENSE file for details.


# Rust DBSCAN - Geo Point Clustering

This is a Rust port of the Go DBSCAN clustering tool. It reads geographic points from CSV files, applies DBSCAN clustering, and filters results to keep only outliers and the first point in each cluster.

## Features

- Reads latitude/longitude points from CSV files
- Applies DBSCAN clustering algorithm with configurable parameters
- Filters results to keep:
  - Outliers (points that don't belong to any cluster)
  - First point in each cluster (removes noise from clusters)
- Preserves original CSV structure in output
- Fully documented with Rust doc comments
- Comprehensive test suite

## Installation

```bash
cd rust_dbscan
cargo build --release
```

## Usage

### Output to stdout (default)
```bash
cargo run --release -- -i points.csv -e 0.1 -m 3
```

This will output filtered points to stdout as a simple list (one point per line: `latitude,longitude`).

### Output to file
```bash
cargo run --release -- -i points.csv -o filtered_points.csv -e 0.1 -m 3
```

### Command-line Options

- `-i, --input`: Input CSV file path (default: `points.csv`)
- `-o, --output`: Output CSV file path (default: empty, outputs to stdout)
- `-e, --eps`: DBSCAN epsilon parameter - clustering radius in kilometers (default: `0.1`)
- `-m, --min-points`: DBSCAN minPoints parameter - minimum number of points in a cluster (default: `3`)
- `-d, --debug`: Enable debug output

## CSV Format

The input CSV file should have at least two columns: `latitude` and `longitude`. The first row can be a header row (will be automatically detected and preserved).

Example:
```csv
latitude,longitude
40.7128,-74.0060
40.7130,-74.0062
40.7132,-74.0064
```

## Algorithm

The program implements DBSCAN (Density-Based Spatial Clustering of Applications with Noise) clustering algorithm:

1. **Clustering**: Groups points that are within `eps` distance of each other and have at least `minPoints` neighbors
2. **Filtering**: After clustering, filters the results to keep:
   - All outlier points (labeled as -1)
   - Only the first point in each cluster (removes subsequent points in the same cluster)

## Development

```bash
# Run tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Check formatting
cargo fmt --check

# Lint with clippy
cargo clippy --all-targets --all-features -- -D warnings

# Build release
cargo build --release
```

## Code Quality

This Rust port follows modern Rust best practices:

- ✅ Comprehensive documentation with `///` doc comments
- ✅ Proper error handling with `Result` types
- ✅ Safe handling of floating-point comparisons (NaN handling)
- ✅ Clear naming conventions (snake_case for functions, PascalCase for types)
- ✅ Type safety (avoiding sentinel values where possible)
- ✅ Performance considerations (minimizing allocations where possible)
- ✅ Full test coverage matching the Go implementation

## CI/CD

This project uses GitHub Actions for:
- **CI**: Automated testing on push/PR (multiple Rust versions, linting, building)
- **Coverage**: Code coverage reporting with cargo-tarpaulin

## License

MIT


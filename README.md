# DBSCAN Geo Point Clustering

A Go program that reads geographic points from a CSV file, applies DBSCAN clustering, and filters the results to keep only outliers and the first point in each cluster.

## Features

- Reads latitude/longitude points from CSV files
- Applies DBSCAN clustering algorithm with configurable parameters
- Filters results to keep:
  - Outliers (points that don't belong to any cluster)
  - First point in each cluster (removes noise from clusters)
- Preserves original CSV structure in output

## Installation

```bash
go mod download
```

## Usage

### Output to stdout (default)
```bash
go run . -input points.csv -eps 0.1 -min-points 3
```

This will output filtered points to stdout as a simple list (one point per line: `latitude,longitude`).

To suppress informational messages, redirect stderr:
```bash
go run . -input points.csv -eps 0.1 -min-points 3 2>/dev/null
```

### Output to file
```bash
go run . -input points.csv -output filtered_points.csv -eps 0.1 -min-points 3
```

### Command-line Options

- `-input`: Input CSV file path (default: `points.csv`)
- `-output`: Output CSV file path (default: empty, outputs to stdout)
- `-eps`: DBSCAN epsilon parameter - clustering radius in kilometers (default: `0.1`)
- `-min-points`: DBSCAN minPoints parameter - minimum number of points in a cluster (default: `3`)

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


## Example

```bash
# Run with default parameters
go run . -input points.csv -output filtered.csv

# Run with custom parameters
go run . -input points.csv -output filtered.csv -eps 0.5 -min-points 5
```

## Building

```bash
# Build the binary
go build -o go_dbscan_filter .

# Or use Make
make build

# Run the program
./go_dbscan_filter -input points.csv -output filtered.csv
```

## Development

```bash
# Run tests
make test

# Run linter
make lint

# Install dependencies
make deps

# Clean build artifacts
make clean
```

## CI/CD

This project uses GitHub Actions for:
- **CI**: Automated testing on push/PR (multiple Go versions, linting, building)
- **Release**: Automated releases using GoReleaser when tags are pushed

To create a release:
```bash
git tag -a v1.0.0 -m "Release v1.0.0"
git push origin v1.0.0
```

## License

MIT

//! DBSCAN geo point clustering tool
//!
//! Reads geographic points from CSV files, applies DBSCAN clustering, and filters
//! results to keep only outliers and the first point in each cluster.

use clap::Parser;
use csv::{ReaderBuilder, WriterBuilder};
use std::fs::File;
use std::path::PathBuf;

mod cluster;

#[cfg(test)]
mod main_test;

use cluster::{Cluster, DBScan, Point, PointList};

const DBSCAN_OUTLIER_INDEX: i32 = -1;

#[derive(Parser)]
#[command(name = "rust_dbscan")]
#[command(about = "DBSCAN geo point clustering tool", long_about = None)]
struct Args {
    /// Input CSV file with latitude,longitude columns
    #[arg(short, long, default_value = "points.csv")]
    input: PathBuf,

    /// Output CSV file with filtered points (default: stdout)
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// DBSCAN epsilon parameter (clustering radius in km)
    #[arg(short, long, default_value_t = 0.1)]
    eps: f64,

    /// DBSCAN minPoints parameter (minimum points in cluster)
    #[arg(short = 'm', long, default_value_t = 3)]
    min_points: usize,

    /// Enable debug output
    #[arg(short, long)]
    debug: bool,
}

fn main() {
    let args = Args::parse();

    // Read points and CSV records from file (read once, reuse for output)
    let (points, csv_records) = match read_points_and_csv(&args.input) {
        Ok(result) => result,
        Err(e) => {
            eprintln!("Error reading CSV: {}", e);
            std::process::exit(1);
        }
    };

    if points.is_empty() {
        eprintln!("No points found in CSV file");
        std::process::exit(1);
    }

    // Debug output (only if debug flag is set)
    if args.debug {
        println!("Read {} points from {:?}", points.len(), args.input);
        println!(
            "Running DBSCAN with eps={:.4} km, minPoints={}",
            args.eps, args.min_points
        );
    }

    // Run DBSCAN clustering
    let (clusters, noise) = DBScan(&points, args.eps, args.min_points);

    if args.debug {
        println!("Found {} clusters", clusters.len());
        println!("Found {} noise points", noise.len());
    }

    // Build labels array from clusters and noise for filtering
    let labels = build_labels(&clusters, &noise, points.len());

    // Filter points based on:
    // 1. Keep outliers (label == -1)
    // 2. Keep first point in each cluster (idx == 0 or label != labels[idx-1])
    let filtered_indices = filter_points(&points, &labels);

    if args.debug {
        println!("Filtered to {} points", filtered_indices.len());
    }

    // Write filtered points to output (stdout or file)
    match args.output {
        None => {
            // Output to stdout as simple list of points
            if let Err(e) = write_filtered_points_to_stdout(&csv_records, &filtered_indices) {
                eprintln!("Error writing to stdout: {}", e);
                std::process::exit(1);
            }
        }
        Some(output_file) => {
            // Write filtered points to output CSV file
            if let Err(e) =
                write_filtered_points_to_csv(&output_file, &csv_records, &filtered_indices)
            {
                eprintln!("Error writing CSV: {}", e);
                std::process::exit(1);
            }
            if args.debug {
                println!("Filtered points written to {:?}", output_file);
            }
        }
    }
}

/// CSV records type alias for readability
type CsvRecords = Vec<Vec<String>>;

/// Reads points and CSV records from a file in a single pass
///
/// Expected format: `latitude,longitude` (header row is optional)
///
/// # Returns
///
/// A tuple `(points, records)` where:
/// - `points` are parsed points for clustering
/// - `records` are raw CSV records for output preservation
fn read_points_and_csv(
    filename: &PathBuf,
) -> Result<(PointList, CsvRecords), Box<dyn std::error::Error>> {
    let file = File::open(filename)?;
    let mut reader = ReaderBuilder::new().has_headers(false).from_reader(file);

    let mut points = PointList::new();
    let mut records = Vec::new();

    // Read all records first
    for result in reader.records() {
        let record = result?;
        let record_vec: Vec<String> = record.iter().map(|s| s.to_string()).collect();
        records.push(record_vec);
    }

    if records.is_empty() {
        return Ok((points, records));
    }

    // Determine if first row is header
    let has_header = records[0][0].parse::<f64>().is_err();

    // Parse points from records
    let start_idx = if has_header { 1 } else { 0 };

    for record in records.iter().skip(start_idx) {
        if record.len() < 2 {
            continue;
        }

        let lat = record[0].parse::<f64>();
        let lon = record[1].parse::<f64>();
        if let (Ok(lat), Ok(lon)) = (lat, lon) {
            // Point is [2]float64 where [0]=Lon, [1]=Lat
            points.push(Point([lon, lat]));
        }
    }

    Ok((points, records))
}

/// Filters points based on the filtering logic:
/// - Keep outliers (label == -1)
/// - Keep first point in each cluster (idx == 0 or label != labels[idx-1])
///
/// Tracks added points by their coordinates to avoid duplicates
fn filter_points(points: &PointList, labels: &[i32]) -> Vec<usize> {
    let mut filtered = Vec::new();
    let mut added = Vec::new(); // Track already added points by coordinates

    for (idx, &label) in labels.iter().enumerate() {
        let point = points[idx];

        // Skip if point with same coordinates already added
        if added.contains(&point) {
            continue;
        }

        // Keep if it's an outlier
        if label == DBSCAN_OUTLIER_INDEX {
            filtered.push(idx);
            added.push(point);
            continue;
        }

        // Keep if it's the first point (idx == 0)
        if idx == 0 {
            filtered.push(idx);
            added.push(point);
            continue;
        }

        // Keep if it's the first point in a cluster (label != previous label)
        if label != labels[idx - 1] {
            filtered.push(idx);
            added.push(point);
        }
    }

    filtered
}

/// Creates a labels array from clusters and noise
///
/// `labels[i]` = cluster ID for point i, or -1 for noise
fn build_labels(clusters: &[Cluster], _noise: &[usize], num_points: usize) -> Vec<i32> {
    let mut labels = vec![-1; num_points];

    // Mark cluster points
    for cluster in clusters {
        for &idx in &cluster.points {
            labels[idx] = cluster.c as i32;
        }
    }

    // Noise points are already -1, but we verify they're in the noise list
    // (they should already be -1 from initialization)

    labels
}

/// Writes filtered points to output CSV
///
/// Uses pre-read CSV records to preserve any additional columns
fn write_filtered_points_to_csv(
    output_file: &PathBuf,
    csv_records: &[Vec<String>],
    filtered_indices: &[usize],
) -> Result<(), Box<dyn std::error::Error>> {
    // Create a set of filtered indices for quick lookup
    let filtered_set: std::collections::HashSet<usize> = filtered_indices.iter().copied().collect();

    // Write filtered records to output
    let out_file = File::create(output_file)?;
    let mut writer = WriterBuilder::new().from_writer(out_file);

    // Determine if first row is header
    let has_header = if !csv_records.is_empty() {
        csv_records[0][0].parse::<f64>().is_err()
    } else {
        false
    };

    if has_header {
        // Write header
        writer.write_record(&csv_records[0])?;
    }

    // Write filtered data rows
    let start_idx = if has_header { 1 } else { 0 };

    for (i, record) in csv_records.iter().enumerate().skip(start_idx) {
        let point_idx = i - start_idx;
        if filtered_set.contains(&point_idx) {
            writer.write_record(record)?;
        }
    }

    writer.flush()?;
    Ok(())
}

/// Writes filtered points to stdout as a simple list
///
/// Format: `latitude,longitude` (one point per line)
///
/// Uses pre-read CSV records to preserve order
fn write_filtered_points_to_stdout(
    csv_records: &[Vec<String>],
    filtered_indices: &[usize],
) -> Result<(), Box<dyn std::error::Error>> {
    // Create a set of filtered indices for quick lookup
    let filtered_set: std::collections::HashSet<usize> = filtered_indices.iter().copied().collect();

    // Determine if first row is header
    let has_header = if !csv_records.is_empty() {
        csv_records[0][0].parse::<f64>().is_err()
    } else {
        false
    };

    // Write filtered points to stdout
    let start_idx = if has_header { 1 } else { 0 };

    for (i, record) in csv_records.iter().enumerate().skip(start_idx) {
        let point_idx = i - start_idx;
        if filtered_set.contains(&point_idx) {
            // Output as: latitude,longitude
            if record.len() >= 2 {
                println!("{},{}", record[0], record[1]);
            }
        }
    }

    Ok(())
}

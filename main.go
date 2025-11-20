// Package main implements a DBSCAN geo point clustering tool that reads
// geographic points from CSV files, applies DBSCAN clustering, and filters
// results to keep only outliers and the first point in each cluster.
package main

import (
	"encoding/csv"
	"flag"
	"fmt"
	"io"
	"log"
	"os"
	"strconv"

	"go-dbscan-filter/internal/cluster"
)

const (
	// DbscanOutlierIndex is the label for noise/outlier points
	DbscanOutlierIndex = -1
)

func main() {
	var (
		inputFile  = flag.String("input", "points.csv", "Input CSV file with latitude,longitude columns")
		outputFile = flag.String("output", "", "Output CSV file with filtered points (default: stdout)")
		eps        = flag.Float64("eps", 0.1, "DBSCAN epsilon parameter (clustering radius in km)")
		minPoints  = flag.Int("min-points", 3, "DBSCAN minPoints parameter (minimum points in cluster)")
		debug      = flag.Bool("debug", false, "Enable debug output")
	)
	flag.Parse()

	// Read points and CSV records from file (read once, reuse for output)
	points, csvRecords, err := readPointsAndCSV(*inputFile)
	if err != nil {
		log.Fatalf("Error reading CSV: %v", err)
	}

	if len(points) == 0 {
		log.Fatalf("No points found in CSV file")
	}

	// Debug output (only if debug flag is set)
	if *debug {
		fmt.Printf("Read %d points from %s\n", len(points), *inputFile)
		fmt.Printf("Running DBSCAN with eps=%.4f km, minPoints=%d\n", *eps, *minPoints)
	}

	// Run DBSCAN clustering
	clusters, noise := cluster.DBScan(points, *eps, *minPoints)

	if *debug {
		fmt.Printf("Found %d clusters\n", len(clusters))
		fmt.Printf("Found %d noise points\n", len(noise))
	}

	// Build labels array from clusters and noise for filtering
	labels := buildLabels(clusters, noise, len(points))

	// Filter points based on Ruby logic:
	// 1. Keep outliers (label == -1)
	// 2. Keep first point in each cluster (idx == 0 or label != labels[idx-1])
	filteredIndices := filterPoints(labels)

	if *debug {
		fmt.Printf("Filtered to %d points\n", len(filteredIndices))
	}

	// Write filtered points to output (stdout or file)
	if *outputFile == "" {
		// Output to stdout as simple list of points
		err = writeFilteredPointsToStdout(csvRecords, filteredIndices)
		if err != nil {
			log.Fatalf("Error writing to stdout: %v", err)
		}
	} else {
		// Write filtered points to output CSV file
		err = writeFilteredPointsToCSV(outputFile, csvRecords, filteredIndices)
		if err != nil {
			log.Fatalf("Error writing CSV: %v", err)
		}
		if *debug {
			fmt.Printf("Filtered points written to %s\n", *outputFile)
		}
	}
}

// readPointsAndCSV reads points and CSV records from a file in a single pass
// Expected format: latitude,longitude (header row is optional)
// Returns points for clustering and raw records for output preservation
func readPointsAndCSV(filename string) (cluster.PointList, [][]string, error) {
	file, err := os.Open(filename)
	if err != nil {
		return nil, nil, err
	}
	defer func() {
		if closeErr := file.Close(); closeErr != nil {
			log.Printf("Error closing file: %v", closeErr)
		}
	}()

	reader := csv.NewReader(file)
	points := cluster.PointList{}
	records := [][]string{}

	// Read all records first
	for {
		record, err := reader.Read()
		if err == io.EOF {
			break
		}
		if err != nil {
			return nil, nil, err
		}
		records = append(records, record)
	}

	if len(records) == 0 {
		return points, records, nil
	}

	// Determine if first row is header
	hasHeader := false
	_, err = strconv.ParseFloat(records[0][0], 64)
	if err != nil {
		hasHeader = true
	}

	// Parse points from records
	startIdx := 0
	if hasHeader {
		startIdx = 1
	}

	for i := startIdx; i < len(records); i++ {
		record := records[i]
		if len(record) < 2 {
			continue
		}

		lat, err1 := strconv.ParseFloat(record[0], 64)
		lon, err2 := strconv.ParseFloat(record[1], 64)
		if err1 != nil || err2 != nil {
			continue // Skip invalid rows
		}

		// Point is [2]float64 where [0]=Lon, [1]=Lat
		points = append(points, cluster.Point{lon, lat})
	}

	return points, records, nil
}

// filterPoints filters points based on the Ruby logic:
// - Keep outliers (label == -1)
// - Keep first point in each cluster (idx == 0 or label != labels[idx-1])
func filterPoints(labels []int) []int {
	filtered := []int{}

	for idx, label := range labels {
		// Keep if it's an outlier
		if label == DbscanOutlierIndex {
			filtered = append(filtered, idx)
			continue
		}

		// Keep if it's the first point (idx == 0)
		if idx == 0 {
			filtered = append(filtered, idx)
			continue
		}

		// Keep if it's the first point in a cluster (label != previous label)
		if label != labels[idx-1] {
			filtered = append(filtered, idx)
		}
	}

	return filtered
}

// buildLabels creates a labels array from clusters and noise
// labels[i] = cluster ID for point i, or -1 for noise
func buildLabels(clusters []cluster.Cluster, _ []int, numPoints int) []int {
	labels := make([]int, numPoints)

	// Initialize all as noise
	for i := range labels {
		labels[i] = -1
	}

	// Mark cluster points
	for _, cluster := range clusters {
		for _, idx := range cluster.Points {
			labels[idx] = cluster.C
		}
	}

	// Noise points are already -1, but we verify they're in the noise list
	// (they should already be -1 from initialization)

	return labels
}

// writeFilteredPointsToCSV writes filtered points to output CSV
// Uses pre-read CSV records to preserve any additional columns
func writeFilteredPointsToCSV(outputFile *string, csvRecords [][]string, filteredIndices []int) error {
	// Create a set of filtered indices for quick lookup
	filteredSet := make(map[int]bool)
	for _, idx := range filteredIndices {
		filteredSet[idx] = true
	}

	// Write filtered records to output
	outFile, err := os.Create(*outputFile)
	if err != nil {
		return err
	}
	defer func() {
		if closeErr := outFile.Close(); closeErr != nil {
			log.Printf("Error closing output file: %v", closeErr)
		}
	}()

	writer := csv.NewWriter(outFile)
	defer writer.Flush()

	// Determine if first row is header
	hasHeader := false
	if len(csvRecords) > 0 {
		_, err := strconv.ParseFloat(csvRecords[0][0], 64)
		if err != nil {
			hasHeader = true
			// Write header
			if err := writer.Write(csvRecords[0]); err != nil {
				return err
			}
		}
	}

	// Write filtered data rows
	startIdx := 0
	if hasHeader {
		startIdx = 1
	}

	for i := startIdx; i < len(csvRecords); i++ {
		pointIdx := i - startIdx
		if filteredSet[pointIdx] {
			if err := writer.Write(csvRecords[i]); err != nil {
				return err
			}
		}
	}

	return nil
}

// writeFilteredPointsToStdout writes filtered points to stdout as a simple list
// Format: latitude,longitude (one point per line)
// Uses pre-read CSV records to preserve order
func writeFilteredPointsToStdout(csvRecords [][]string, filteredIndices []int) error {
	// Create a set of filtered indices for quick lookup
	filteredSet := make(map[int]bool)
	for _, idx := range filteredIndices {
		filteredSet[idx] = true
	}

	// Determine if first row is header
	hasHeader := false
	if len(csvRecords) > 0 {
		_, err := strconv.ParseFloat(csvRecords[0][0], 64)
		if err != nil {
			hasHeader = true
		}
	}

	// Write filtered points to stdout
	startIdx := 0
	if hasHeader {
		startIdx = 1
	}

	for i := startIdx; i < len(csvRecords); i++ {
		pointIdx := i - startIdx
		if filteredSet[pointIdx] {
			// Output as: latitude,longitude
			if len(csvRecords[i]) >= 2 {
				fmt.Printf("%s,%s\n", csvRecords[i][0], csvRecords[i][1])
			}
		}
	}

	return nil
}

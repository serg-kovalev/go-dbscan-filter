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

	// Read points from CSV
	points, err := readPointsFromCSV(*inputFile)
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
		err = writeFilteredPointsToStdout(*inputFile, filteredIndices, points)
		if err != nil {
			log.Fatalf("Error writing to stdout: %v", err)
		}
	} else {
		// Write filtered points to output CSV file
		err = writeFilteredPointsToCSV(*inputFile, *outputFile, filteredIndices, points)
		if err != nil {
			log.Fatalf("Error writing CSV: %v", err)
		}
		if *debug {
			fmt.Printf("Filtered points written to %s\n", *outputFile)
		}
	}
}

// readPointsFromCSV reads points from a CSV file
// Expected format: latitude,longitude (header row is optional)
func readPointsFromCSV(filename string) (cluster.PointList, error) {
	file, err := os.Open(filename)
	if err != nil {
		return nil, err
	}
	defer func() {
		if closeErr := file.Close(); closeErr != nil {
			log.Printf("Error closing file: %v", closeErr)
		}
	}()

	reader := csv.NewReader(file)
	points := cluster.PointList{}

	// Read header (if present, we'll skip it if it's not numeric)
	firstRow, err := reader.Read()
	if err != nil {
		return nil, err
	}

	// Try to parse first row as numbers, if it fails, it's a header
	_, err = strconv.ParseFloat(firstRow[0], 64)
	if err == nil {
		// First row is data, add it
		if len(firstRow) >= 2 {
			lat, err1 := strconv.ParseFloat(firstRow[0], 64)
			lon, err2 := strconv.ParseFloat(firstRow[1], 64)
			if err1 == nil && err2 == nil {
				// Point is [2]float64 where [0]=Lon, [1]=Lat
				points = append(points, cluster.Point{lon, lat})
			}
		}
	}
	// If err != nil, first row is header, continue

	// Read remaining rows
	for {
		record, err := reader.Read()
		if err == io.EOF {
			break
		}
		if err != nil {
			return nil, err
		}

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

	return points, nil
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
// It reads the original CSV to preserve any additional columns
func writeFilteredPointsToCSV(inputFile, outputFile string, filteredIndices []int, _ cluster.PointList) error {
	// Create a set of filtered indices for quick lookup
	filteredSet := make(map[int]bool)
	for _, idx := range filteredIndices {
		filteredSet[idx] = true
	}

	// Read original CSV
	inFile, err := os.Open(inputFile)
	if err != nil {
		return err
	}
	defer func() {
		if closeErr := inFile.Close(); closeErr != nil {
			log.Printf("Error closing input file: %v", closeErr)
		}
	}()

	reader := csv.NewReader(inFile)
	records := [][]string{}

	// Read all records
	for {
		record, err := reader.Read()
		if err == io.EOF {
			break
		}
		if err != nil {
			return err
		}
		records = append(records, record)
	}

	// Write filtered records to output
	outFile, err := os.Create(outputFile)
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
	if len(records) > 0 {
		_, err := strconv.ParseFloat(records[0][0], 64)
		if err != nil {
			hasHeader = true
			// Write header
			if err := writer.Write(records[0]); err != nil {
				return err
			}
		}
	}

	// Write filtered data rows
	startIdx := 0
	if hasHeader {
		startIdx = 1
	}

	for i := startIdx; i < len(records); i++ {
		pointIdx := i - startIdx
		if filteredSet[pointIdx] {
			if err := writer.Write(records[i]); err != nil {
				return err
			}
		}
	}

	return nil
}

// writeFilteredPointsToStdout writes filtered points to stdout as a simple list
// Format: latitude,longitude (one point per line)
func writeFilteredPointsToStdout(inputFile string, filteredIndices []int, _ cluster.PointList) error {
	// Create a set of filtered indices for quick lookup
	filteredSet := make(map[int]bool)
	for _, idx := range filteredIndices {
		filteredSet[idx] = true
	}

	// Read original CSV to preserve order and any additional columns
	inFile, err := os.Open(inputFile)
	if err != nil {
		return err
	}
	defer func() {
		if closeErr := inFile.Close(); closeErr != nil {
			log.Printf("Error closing input file: %v", closeErr)
		}
	}()

	reader := csv.NewReader(inFile)
	records := [][]string{}

	// Read all records
	for {
		record, err := reader.Read()
		if err == io.EOF {
			break
		}
		if err != nil {
			return err
		}
		records = append(records, record)
	}

	// Determine if first row is header
	hasHeader := false
	if len(records) > 0 {
		_, err := strconv.ParseFloat(records[0][0], 64)
		if err != nil {
			hasHeader = true
		}
	}

	// Write filtered points to stdout
	startIdx := 0
	if hasHeader {
		startIdx = 1
	}

	for i := startIdx; i < len(records); i++ {
		pointIdx := i - startIdx
		if filteredSet[pointIdx] {
			// Output as: latitude,longitude
			if len(records[i]) >= 2 {
				fmt.Printf("%s,%s\n", records[i][0], records[i][1])
			}
		}
	}

	return nil
}

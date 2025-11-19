package main

import (
	"os"
	"testing"

	"go-dbscan-filter/internal/cluster"
)

func TestMainProgram(t *testing.T) {
	// Create a test CSV file
	testCSV := `latitude,longitude
40.7128,-74.0060
40.7130,-74.0062
40.7132,-74.0064
40.7500,-73.9900
40.7502,-73.9902
40.7504,-73.9904
40.8000,-73.9500
41.0000,-74.0000`

	err := os.WriteFile("test_points.csv", []byte(testCSV), 0644)
	if err != nil {
		t.Fatalf("Failed to create test CSV: %v", err)
	}
	defer os.Remove("test_points.csv")

	// Read points
	points, err := readPointsFromCSV("test_points.csv")
	if err != nil {
		t.Fatalf("Failed to read CSV: %v", err)
	}

	if len(points) != 8 {
		t.Errorf("Expected 8 points, got %d", len(points))
	}

	// Test DBSCAN
	clusters, noise := cluster.DBScan(points, 0.1, 3)

	if len(clusters) == 0 {
		t.Error("Expected at least one cluster")
	}

	// Build labels and test filtering
	labels := buildLabels(clusters, noise, len(points))
	filteredIndices := filterPoints(labels)

	// Verify filtering logic:
	// 1. All outliers should be included
	for _, noiseIdx := range noise {
		found := false
		for _, idx := range filteredIndices {
			if idx == noiseIdx {
				found = true
				break
			}
		}
		if !found {
			t.Errorf("Noise point at index %d should be in filtered results", noiseIdx)
		}
	}

	// 2. First point in each cluster should be included
	for _, cluster := range clusters {
		if len(cluster.Points) > 0 {
			firstPoint := cluster.Points[0]
			found := false
			for _, idx := range filteredIndices {
				if idx == firstPoint {
					found = true
					break
				}
			}
			if !found {
				t.Errorf("First point of cluster %d (index %d) should be in filtered results", cluster.C, firstPoint)
			}
		}
	}

	// 3. First point overall should be included
	if len(filteredIndices) == 0 || filteredIndices[0] != 0 {
		// Check if 0 is in filtered indices
		found := false
		for _, idx := range filteredIndices {
			if idx == 0 {
				found = true
				break
			}
		}
		if !found && labels[0] != -1 {
			t.Error("First point (index 0) should be in filtered results if it's not noise")
		}
	}

	t.Logf("Test passed: %d clusters, %d noise points, %d filtered points", len(clusters), len(noise), len(filteredIndices))
}

func TestFilterPointsLogic(t *testing.T) {
	// Test the Ruby-style filtering logic
	tests := []struct {
		name           string
		labels         []int
		expectedCount  int
		expectedIndices []int
	}{
		{
			name:           "all outliers",
			labels:         []int{-1, -1, -1},
			expectedCount:  3,
			expectedIndices: []int{0, 1, 2},
		},
		{
			name:           "single cluster",
			labels:         []int{0, 0, 0},
			expectedCount:  1,
			expectedIndices: []int{0},
		},
		{
			name:           "two clusters",
			labels:         []int{0, 0, 1, 1},
			expectedCount:  2,
			expectedIndices: []int{0, 2},
		},
		{
			name:           "mixed outliers and clusters",
			labels:         []int{-1, 0, 0, -1, 1, 1},
			expectedCount:  4,
			expectedIndices: []int{0, 1, 3, 4},
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			result := filterPoints(tt.labels)
			if len(result) != tt.expectedCount {
				t.Errorf("Expected %d filtered points, got %d", tt.expectedCount, len(result))
			}
			for i, expectedIdx := range tt.expectedIndices {
				if i < len(result) && result[i] != expectedIdx {
					t.Errorf("Expected index %d at position %d, got %d", expectedIdx, i, result[i])
				}
			}
		})
	}
}


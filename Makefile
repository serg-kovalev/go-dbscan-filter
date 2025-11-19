.PHONY: build test lint clean install

# Build the application
build:
	go build -buildvcs=false -o go_dbscan_filter .

# Run tests
test:
	go test -v -race -coverprofile=coverage.out ./...
	go tool cover -html=coverage.out -o coverage.html

# Run tests without race detector (faster)
test-fast:
	go test -v ./...

# Run linter
lint:
	golangci-lint run

# Clean build artifacts
clean:
	rm -f go_dbscan_filter go_dbscan_filter.exe
	rm -f coverage.out coverage.html
	rm -rf dist/

# Install dependencies
deps:
	go mod download
	go mod tidy

# Install the binary
install:
	go install .

# Run the program with example data
run:
	go run . -input points.csv -eps 0.1 -min-points 3

# Run with debug
run-debug:
	go run . -input points.csv -eps 0.1 -min-points 3 -debug

# Build for multiple platforms
build-all:
	GOOS=linux GOARCH=amd64 go build -o dist/go_dbscan_filter-linux-amd64 .
	GOOS=linux GOARCH=arm64 go build -o dist/go_dbscan_filter-linux-arm64 .
	GOOS=darwin GOARCH=amd64 go build -o dist/go_dbscan_filter-darwin-amd64 .
	GOOS=darwin GOARCH=arm64 go build -o dist/go_dbscan_filter-darwin-arm64 .
	GOOS=windows GOARCH=amd64 go build -o dist/go_dbscan_filter-windows-amd64.exe .
	GOOS=windows GOARCH=arm64 go build -o dist/go_dbscan_filter-windows-arm64.exe .


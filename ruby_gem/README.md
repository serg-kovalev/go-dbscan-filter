# GoDbscanFilterBinary

A Ruby gem that provides the `go_dbscan_filter` binary for DBSCAN geo point clustering. The gem automatically downloads the correct platform-specific binary during installation.

## Installation

Add this line to your application's Gemfile:

```ruby
gem 'go_dbscan_filter_binary'
```

And then execute:

```bash
$ bundle install
```

Or install it yourself as:

```bash
$ gem install go_dbscan_filter_binary
```

## Usage

After installation, you can access the binary path:

```ruby
require 'go_dbscan_filter_binary'

# Get the binary path
binary_path = GoDbscanFilterBinary.binary_path

# Check if binary exists
if GoDbscanFilterBinary.binary_exists?
  system("#{GoDbscanFilterBinary.binary!} -input points.csv -output filtered.csv")
end

# Or use the executable directly
system("go_dbscan_filter -input points.csv -output filtered.csv")
```

## Supported Platforms

- Linux (amd64, arm64)
- macOS (darwin) (amd64, arm64)
- Windows (amd64, arm64)

The gem automatically detects your platform and downloads the appropriate binary from GitHub releases.

## Development

After checking out the repo, run `bundle install` to install dependencies. Then, run `rake build` to build the gem.

To install this gem onto your local machine, run `bundle exec rake install`.

## License

The gem is available as open source under the terms of the [MIT License](LICENSE).


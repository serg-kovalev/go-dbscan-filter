# frozen_string_literal: true

require 'fileutils'
require 'net/http'
require 'uri'
require 'json'
require 'zlib'
require 'stringio'
require 'tempfile'
require 'open-uri'

# Determine platform
def detect_platform
  os = case RbConfig::CONFIG['host_os']
  when /linux/i
    'linux'
  when /darwin|mac os/i
    'darwin'
  when /mswin|msys|mingw|cygwin|bccwin|wince|emc/i
    'windows'
  else
    raise "Unsupported OS: #{RbConfig::CONFIG['host_os']}"
  end

  arch = case RbConfig::CONFIG['host_cpu']
  when /x86_64|amd64/i
    'amd64'
  when /arm64|aarch64/i
    'arm64'
  else
    raise "Unsupported architecture: #{RbConfig::CONFIG['host_cpu']}"
  end

  [os, arch]
end

# Get the gem version (should match the release version)
def get_gem_version
  # When installed, extconf.rb runs in: gem_dir/ext/go_dbscan_filter_binary/extconf.rb
  # We need to find: gem_dir/lib/go_dbscan_filter_binary/version.rb

  # Calculate gem root: go up two directory levels from extconf.rb
  # Use expand_path to ensure we get absolute path
  extconf_dir = File.expand_path(File.dirname(__FILE__))
  # extconf_dir = gem_dir/ext/go_dbscan_filter_binary
  ext_dir = File.expand_path('..', extconf_dir)
  # ext_dir = gem_dir/ext
  gem_root = File.expand_path('..', ext_dir)
  # gem_root = gem_dir

  version_file = File.join(gem_root, 'lib', 'go_dbscan_filter_binary', 'version.rb')

  # Try to load the version file directly
  if File.exist?(version_file)
    version_content = File.read(version_file)
    if version_content =~ /VERSION\s*=\s*['"]([^'"]+)['"]/
      return $1
    end
  end

  # Fallback: try to require it
  begin
    $LOAD_PATH.unshift(File.join(gem_root, 'lib'))
    require 'go_dbscan_filter_binary/version'
    return GoDbscanFilterBinary::VERSION
  rescue LoadError => e
    raise "Could not determine gem version. Tried: #{version_file} (gem_root: #{gem_root}, extconf_dir: #{extconf_dir})"
  end
end

# Extract tar.gz using system tar command (more reliable)
def extract_tar_gz(data, binary_name, output_path)
  require 'tempfile'
  temp_tar = Tempfile.new(['binary', '.tar.gz'])
  temp_tar.binmode
  temp_tar.write(data)
  temp_tar.close

  # Create temp directory for extraction
  temp_dir = Dir.mktmpdir('go_dbscan_filter_extract')
  begin
    # Extract entire archive to temp directory
    if system("tar -xzf '#{temp_tar.path}' -C '#{temp_dir}' 2>/dev/null")
      # Find the binary in the extracted files (could be at root or in subdirectory)
      found_binary = nil
      Dir.glob(File.join(temp_dir, '**', binary_name)).each do |file|
        if File.file?(file) && File.executable?(file) || true # Accept any file matching the name
          found_binary = file
          break
        end
      end

      # Also check root level
      root_binary = File.join(temp_dir, binary_name)
      found_binary = root_binary if File.exist?(root_binary) && !found_binary

      if found_binary && File.exist?(found_binary)
        FileUtils.cp(found_binary, output_path)
        File.chmod(0o755, output_path)
        temp_tar.unlink
        return true
      end
    end
  ensure
    FileUtils.rm_rf(temp_dir) if Dir.exist?(temp_dir)
  end

  # Fallback: manual extraction
  io = StringIO.new(data)
  gz = Zlib::GzipReader.new(io)
  tar_data = gz.read
  gz.close

  # Simple tar extraction
  pos = 0
  while pos < tar_data.length
    header = tar_data[pos, 512]
    break if header.nil? || header.empty? || header[0] == "\0"

    name = header[0, 100].strip
    size_str = header[124, 12].strip
    size = size_str.oct rescue 0

    pos += 512

    if name.end_with?(binary_name) && size > 0
      binary_data = tar_data[pos, size]
      File.open(output_path, 'wb') do |f|
        f.write(binary_data)
      end
      File.chmod(0o755, output_path)
      temp_tar.unlink
      return true
    end

    pos += (size + 511) / 512 * 512
  end

  temp_tar.unlink
  false
end

# Extract zip using system unzip command (more reliable)
def extract_zip(data, binary_name, output_path)
  require 'tempfile'
  temp_zip = Tempfile.new(['binary', '.zip'])
  temp_zip.binmode
  temp_zip.write(data)
  temp_zip.close

  # Create temp directory for extraction
  temp_dir = Dir.mktmpdir('go_dbscan_filter_extract')
  begin
    # Extract entire archive to temp directory
    if system("unzip -q '#{temp_zip.path}' -d '#{temp_dir}' 2>/dev/null")
      # Find the binary in the extracted files
      found_binary = nil
      Dir.glob(File.join(temp_dir, '**', binary_name)).each do |file|
        if File.file?(file)
          found_binary = file
          break
        end
      end

      # Also check root level
      root_binary = File.join(temp_dir, binary_name)
      found_binary = root_binary if File.exist?(root_binary) && !found_binary

      if found_binary && File.exist?(found_binary)
        FileUtils.cp(found_binary, output_path)
        temp_zip.unlink
        return true
      end
    end

    # Fallback: try direct extraction to output directory
    if system("unzip -q -j '#{temp_zip.path}' '*#{binary_name}' -d '#{File.dirname(output_path)}' 2>/dev/null")
      result = File.exist?(output_path)
      temp_zip.unlink
      return result
    end
  ensure
    FileUtils.rm_rf(temp_dir) if Dir.exist?(temp_dir)
  end

  temp_zip.unlink
  false
end

# Download and extract the binary
def download_binary(os, arch, version)
  puts "Downloading go_dbscan_filter binary for #{os}/#{arch}..."

  # Construct download URL
  archive_name = "go-dbscan-filter_#{os}_#{arch}"
  archive_name += '.zip' if os == 'windows'
  archive_name += '.tar.gz' unless os == 'windows'

  url = "https://github.com/serg-kovalev/go-dbscan-filter/releases/download/v#{version}/#{archive_name}"

  puts "Downloading from: #{url}"

  # Download the archive (open-uri handles redirects automatically)
  begin
    response_body = URI.open(url, 'User-Agent' => 'go_dbscan_filter_binary-gem', &:read)
  rescue OpenURI::HTTPError => e
    raise "Failed to download binary: HTTP #{e.io.status[0]} #{e.io.status[1]}"
  rescue => e
    raise "Failed to download binary: #{e.message}"
  end

  # Determine binary name
  binary_name = 'go_dbscan_filter'
  binary_name += '.exe' if os == 'windows'

  # Download binary to lib/go_dbscan_filter_binary/bin/ (wrapper script stays in bin/)
  # From ext/go_dbscan_filter_binary/extconf.rb, go up to gem root
  gem_root = File.expand_path('../..', File.dirname(__FILE__))
  bin_dir = File.join(gem_root, 'lib', 'go_dbscan_filter_binary', 'bin')
  FileUtils.mkdir_p(bin_dir)
  output_path = File.join(bin_dir, binary_name)

  # Extract binary
  success = if os == 'windows'
    extract_zip(response_body, binary_name, output_path)
  else
    extract_tar_gz(response_body, binary_name, output_path)
  end

  unless success && File.exist?(output_path)
    raise "Binary #{binary_name} was not extracted correctly"
  end

  puts "Binary downloaded successfully to #{output_path}"
  output_path
end

# Main installation logic
begin
  os, arch = detect_platform
  version = get_gem_version

  puts "Installing go_dbscan_filter binary for platform: #{os}/#{arch}"
  puts "Gem version: #{version} (will download matching release v#{version})"

  binary_path = download_binary(os, arch, version)

  # Determine binary name for Makefile
  binary_name = 'go_dbscan_filter'
  binary_name += '.exe' if os == 'windows'

  # Get bin directory path (actual binary is in lib/go_dbscan_filter_binary/bin/)
  gem_root = File.expand_path('../..', File.dirname(__FILE__))
  bin_dir = File.join(gem_root, 'lib', 'go_dbscan_filter_binary', 'bin')
  bin_path = File.join(bin_dir, binary_name)

  # Create Makefile (required by RubyGems)
  File.open('Makefile', 'w') do |f|
    f.puts <<~MAKEFILE
      # Makefile for go_dbscan_filter_binary extension
      # The binary is already downloaded by extconf.rb

      .PHONY: all install clean

      all:
      \t@echo "Binary already prepared by extconf.rb"
      \t@test -f "#{bin_path}" || (echo "Error: Binary not found at #{bin_path}" && exit 1)
      \t@echo "Binary verified: #{bin_path}"

      install: all
      \t@echo "Installing go_dbscan_filter binary..."
      \t@test -f "#{bin_path}" || (echo "Error: Binary not found at #{bin_path}" && exit 1)
      \t@chmod +x "#{bin_path}" 2>/dev/null || true
      \t@echo "Binary installed successfully: #{bin_path}"

      clean:
      \t@echo "Cleaning up..."
      \t@# Don't remove Makefile - it's needed for make install
    MAKEFILE
  end

rescue => e
  $stderr.puts "Error installing binary: #{e.message}"
  $stderr.puts e.backtrace.join("\n")
  exit 1
end


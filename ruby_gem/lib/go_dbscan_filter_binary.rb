# frozen_string_literal: true

require_relative 'go_dbscan_filter_binary/version'

module GoDbscanFilterBinary
  class Error < StandardError; end

  # Returns the path to the go_dbscan_filter binary
  def self.binary_path
    @binary_path ||= begin
      # Find the gem's root directory
      # __dir__ points to lib/go_dbscan_filter_binary, so go up two levels to gem root
      gem_root = File.expand_path('../..', __dir__)

      # Construct binary path (actual binary is in lib/go_dbscan_filter_binary/bin/)
      binary_name = 'go_dbscan_filter'
      binary_name += '.exe' if Gem.win_platform?
      bin_path = File.join(__dir__, 'bin', binary_name)

      # Verify the path exists, if not try alternative methods
      unless File.exist?(bin_path)
        # Try using Gem to find the gem path (works for installed gems)
        begin
          spec = Gem::Specification.find_by_name('go_dbscan_filter_binary')
          if spec
            bin_path = File.join(spec.gem_dir, 'lib', 'go_dbscan_filter_binary', 'bin', binary_name)
          end
        rescue Gem::LoadError
          # Gem not found via Gem, try Bundler
          begin
            require 'bundler'
            if defined?(Bundler)
              spec = Bundler.rubygems.find_name('go_dbscan_filter_binary').first
              if spec
                bin_path = File.join(spec.full_gem_path, 'lib', 'go_dbscan_filter_binary', 'bin', binary_name)
              end
            end
          rescue
            # Fallback to original calculation
          end
        end
      end

      bin_path
    end
  end

  # Returns true if the binary exists
  def self.binary_exists?
    File.exist?(binary_path) && File.executable?(binary_path)
  end

  # Returns the binary path or raises an error if it doesn't exist
  def self.binary!
    raise Error, "go_dbscan_filter binary not found at #{binary_path}" unless binary_exists?
    binary_path
  end
end


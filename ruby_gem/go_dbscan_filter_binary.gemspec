# frozen_string_literal: true

require_relative 'lib/go_dbscan_filter_binary/version'

Gem::Specification.new do |spec|
  spec.name          = 'go_dbscan_filter_binary'
  spec.version       = GoDbscanFilterBinary::VERSION
  spec.authors       = ['Sergey Kovalev']
  spec.email         = ['serg-kovalev@users.noreply.github.com>']

  spec.summary       = 'DBSCAN geo point clustering binary'
  spec.description   = 'Provides the go-dbscan-filter binary to be used by other gems'
  spec.homepage      = 'https://github.com/serg-kovalev/go-dbscan-filter'
  spec.license       = 'MIT'

  spec.files         = Dir['lib/**/*', 'ext/**/*', 'bin/**/*', '*.md', 'LICENSE', '*.gemspec']
  # Note: bin/go_dbscan_filter is a Ruby wrapper script that calls the actual Go binary
  # The actual Go binary is downloaded to lib/go_dbscan_filter_binary/bin/ during installation via extconf.rb
  spec.bindir        = 'bin'
  spec.executables   = ['go_dbscan_filter']
  # The executable is the actual Go binary downloaded during installation
  spec.require_paths = ['lib']
  spec.extensions    = ['ext/go_dbscan_filter_binary/extconf.rb']

  spec.required_ruby_version = '>= 2.7.0'

  spec.add_development_dependency 'bundler', '~> 2.0'
  spec.add_development_dependency 'rake', '~> 13.0'
end


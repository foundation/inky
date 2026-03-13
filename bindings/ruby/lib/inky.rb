# frozen_string_literal: true

require "fiddle"
require "fiddle/import"
require "json"

# Inky — Transform email templates into email-safe HTML.
#
# Powered by Rust via Fiddle FFI.
module Inky
  VERSION = "2.0.0"

  module Native
    extend Fiddle::Importer

    LIB_NAMES = case RUBY_PLATFORM
                when /darwin/  then ["libinky.dylib"]
                when /mingw|mswin/ then ["inky.dll"]
                else ["libinky.so"]
                end

    def self.find_library
      name = LIB_NAMES.first
      candidates = [
        # Development: cargo build output
        File.join(__dir__, "..", "..", "..", "target", "release", name),
        File.join(__dir__, "..", "..", "..", "target", "debug", name),
        # Bundled with gem
        File.join(__dir__, name),
        # System paths
        File.join("/usr/local/lib", name),
        File.join("/usr/lib", name),
      ]

      candidates.each do |path|
        resolved = File.expand_path(path)
        return resolved if File.exist?(resolved)
      end

      nil
    end

    lib_path = find_library
    raise "Could not find libinky shared library. Build it with: cargo build -p inky-ffi --release" unless lib_path

    dlload lib_path

    extern "char* inky_transform(const char*)"
    extern "char* inky_transform_with_columns(const char*, unsigned int)"
    extern "char* inky_transform_inline(const char*)"
    extern "char* inky_migrate(const char*)"
    extern "char* inky_migrate_with_details(const char*)"
    extern "char* inky_validate(const char*)"
    extern "char* inky_version()"
    extern "void inky_free(char*)"
  end

  # Transform Inky HTML into email-safe table markup.
  #
  # @param html [String] Inky template HTML
  # @param columns [Integer] Number of grid columns (default: 12)
  # @return [String] Transformed HTML
  def self.transform(html, columns: 12)
    if columns != 12
      Native.inky_transform_with_columns(html, columns).to_s
    else
      Native.inky_transform(html).to_s
    end
  end

  # Transform Inky HTML and inline CSS from <style> blocks.
  #
  # @param html [String] Inky template HTML with <style> blocks
  # @return [String] Transformed HTML with CSS inlined
  def self.transform_inline(html)
    Native.inky_transform_inline(html).to_s
  end

  # Migrate v1 Inky syntax to v2.
  #
  # @param html [String] v1 Inky template HTML
  # @return [String] Migrated v2 HTML
  def self.migrate(html)
    Native.inky_migrate(html).to_s
  end

  # Migrate v1 syntax and return detailed results.
  #
  # @param html [String] v1 Inky template HTML
  # @return [Hash] Hash with :html and :changes keys
  def self.migrate_with_details(html)
    json = Native.inky_migrate_with_details(html).to_s
    JSON.parse(json, symbolize_names: true)
  end

  # Validate an Inky template and return diagnostics.
  #
  # @param html [String] Inky template HTML
  # @return [Array<Hash>] Array of hashes with :severity, :rule, :message keys
  def self.validate(html)
    json = Native.inky_validate(html).to_s
    JSON.parse(json, symbolize_names: true)
  end

  # Get the Inky engine version.
  #
  # @return [String] Version string
  def self.version
    Native.inky_version().to_s
  end
end

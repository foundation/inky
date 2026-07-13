# frozen_string_literal: true

require "fiddle"
require "fiddle/import"
require "json"

# Inky — Transform email templates into email-safe HTML.
#
# Powered by Rust via Fiddle FFI.
module Inky
  VERSION = "2.0.0"

  # Raised when the native inky library reports an error.
  class Error < StandardError; end

  # Raised when the full build pipeline fails.
  class BuildError < Error
    # @return [Array<String>] Non-fatal notes collected before the failure.
    attr_reader :warnings

    def initialize(message, warnings = [])
      super(message)
      @warnings = warnings
    end
  end

  # Result of a full pipeline build.
  BuildResult = Struct.new(:html, :text, :warnings, keyword_init: true)

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
    extern "char* inky_build(const char*, const char*, const char*)"
    extern "void inky_free(char*)"
  end

  # Copy and free a char* result from libinky. A null pointer signals an
  # internal engine error (or null input).
  def self.string_result(ptr)
    raise Error, "inky native call failed (null result)" if ptr.null?

    begin
      ptr.to_s
    ensure
      Native.inky_free(ptr)
    end
  end
  private_class_method :string_result

  def self.check_html!(html)
    raise TypeError, "html must be a String, got #{html.class}" unless html.is_a?(String)
  end
  private_class_method :check_html!

  # Transform Inky HTML into email-safe table markup.
  #
  # @param html [String] Inky template HTML
  # @param columns [Integer] Number of grid columns (default: 12)
  # @return [String] Transformed HTML
  def self.transform(html, columns: 12)
    check_html!(html)
    ptr = if columns != 12
            Native.inky_transform_with_columns(html, columns)
          else
            Native.inky_transform(html)
          end
    string_result(ptr)
  end

  # Transform Inky HTML and inline CSS from <style> blocks.
  #
  # @param html [String] Inky template HTML with <style> blocks
  # @return [String] Transformed HTML with CSS inlined
  def self.transform_inline(html)
    check_html!(html)
    string_result(Native.inky_transform_inline(html))
  end

  # Migrate v1 Inky syntax to v2.
  #
  # @param html [String] v1 Inky template HTML
  # @return [String] Migrated v2 HTML
  def self.migrate(html)
    check_html!(html)
    string_result(Native.inky_migrate(html))
  end

  # Migrate v1 syntax and return detailed results.
  #
  # @param html [String] v1 Inky template HTML
  # @return [Hash] Hash with :html and :changes keys
  def self.migrate_with_details(html)
    check_html!(html)
    json = string_result(Native.inky_migrate_with_details(html))
    JSON.parse(json, symbolize_names: true)
  end

  # Validate an Inky template and return diagnostics.
  #
  # @param html [String] Inky template HTML
  # @return [Array<Hash>] Array of hashes with :severity, :rule, :message keys
  def self.validate(html)
    check_html!(html)
    json = string_result(Native.inky_validate(html))
    JSON.parse(json, symbolize_names: true)
  end

  # Get the Inky engine version.
  #
  # @return [String] Version string
  def self.version
    string_result(Native.inky_version())
  end

  # Run the full build pipeline: layouts, includes, custom components,
  # data merge, framework SCSS, component transform, CSS inlining, and
  # output cleanup — identical to `inky build`.
  #
  # @param html [String] Inky template HTML
  # @param base_path [String, nil] Directory used to resolve layouts,
  #   includes, custom components, and linked SCSS/CSS
  # @param options [Hash] inline_css, framework_css, components_dir,
  #   columns, hybrid, bulletproof_buttons, plain_text, data (Hash of
  #   merge variables)
  # @return [BuildResult]
  # @raise [BuildError] if the pipeline fails; carries #warnings
  def self.build(html, base_path: nil, **options)
    check_html!(html)
    options_json = JSON.generate(options)

    json = string_result(Native.inky_build(html, base_path, options_json))
    envelope = JSON.parse(json, symbolize_names: true)

    warnings = envelope[:warnings] || []
    unless envelope[:ok]
      raise BuildError.new(envelope[:error] || "unknown build error", warnings)
    end

    BuildResult.new(html: envelope[:html], text: envelope[:text], warnings: warnings)
  end
end

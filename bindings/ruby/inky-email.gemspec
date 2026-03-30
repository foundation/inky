Gem::Specification.new do |s|
  s.name        = "inky-email"
  s.version     = "2.0.0-beta.5"
  s.summary     = "Transform email templates into email-safe HTML"
  s.description = "Inky converts simple HTML with custom components into email-safe table markup. Powered by Rust via Fiddle."
  s.authors     = ["ZURB"]
  s.license     = "MIT"
  s.homepage    = "https://github.com/foundation/inky"
  s.files       = ["lib/inky.rb"]
  s.require_paths = ["lib"]
  s.required_ruby_version = ">= 2.7.0"

  s.metadata = {
    "source_code_uri" => "https://github.com/foundation/inky",
    "bug_tracker_uri" => "https://github.com/foundation/inky/issues",
  }
end

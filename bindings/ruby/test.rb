# Tests for the Inky Ruby gem.
# Run: ruby test.rb (after building libinky with `cargo build -p inky-ffi --release`)

require_relative "lib/inky"

$passed = 0
$failed = 0

def assert_true(name, condition, detail = nil)
  if condition
    $passed += 1
  else
    $failed += 1
    puts "  FAIL: #{name}"
    puts "        #{detail}" if detail
  end
end

def assert_equal(name, actual, expected)
  if actual == expected
    $passed += 1
  else
    $failed += 1
    puts "  FAIL: #{name}"
    puts "    expected: #{expected}"
    puts "    got:      #{actual}"
  end
end

# --- transform ---

puts "transform:"

result = Inky.transform('<button href="#">Click</button>')
assert_true "button produces table", result.include?('class="button"')
assert_true "button has href", result.include?('href="#"')
assert_true "button has role=presentation", result.include?('role="presentation"')

result = Inky.transform("<container><row><column>Content</column></row></container>")
assert_true "full layout transforms", result.include?('class="container"')
assert_true "row transforms", result.include?('class="row"')
assert_true "column transforms", result.include?("columns")

result = Inky.transform('<button href="#" size="small" color="alert">Click</button>')
assert_true "v2 size attribute becomes class", result.include?("small")
assert_true "v2 color attribute becomes class", result.include?("alert")

result = Inky.transform('<spacer height="20"></spacer>')
assert_true "spacer with v2 height", result.include?('height="20"')

result = Inky.transform("<divider></divider>")
assert_true "divider transforms", result.include?('class="divider"')

result = Inky.transform('<image src="hero.jpg" alt="Hero" width="600">')
assert_true "image transforms to img", result.include?("<img")
assert_true "image has width", result.include?('width="600"')

result = Inky.transform("<outlook><p>MSO only</p></outlook>")
assert_true "outlook conditional", result.include?("<!--[if mso]>")

result = Inky.transform("<not-outlook><p>Modern</p></not-outlook>")
assert_true "not-outlook conditional", result.include?("<!--[if !mso]><!-->")

# --- transform_inline ---

puts "transform_inline:"

result = Inky.transform_inline(
  '<html><head><style>.button { background: red; }</style></head>' \
  '<body><button href="#">Click</button></body></html>'
)
assert_true "inlines CSS", result.include?("background")
assert_true "transforms components", result.include?('role="presentation"')

# --- transform with columns ---

puts "transform with columns:"

result = Inky.transform("<column>Content</column>", columns: 16)
assert_true "custom column count", result.include?("small-16") || result.include?("large-16")

# --- migrate ---

puts "migrate:"

result = Inky.migrate('<columns large="6" small="12">Content</columns>')
assert_equal "columns to column", result, '<column lg="6" sm="12">Content</column>'

result = Inky.migrate("<h-line></h-line>")
assert_equal "h-line to divider", result, "<divider></divider>"

result = Inky.migrate('<spacer size="16"></spacer>')
assert_equal "spacer size to height", result, '<spacer height="16"></spacer>'

# --- migrate_with_details ---

puts "migrate_with_details:"

result = Inky.migrate_with_details('<columns large="6">Content</columns>')
assert_true "returns hash", result.is_a?(Hash)
assert_true "has html field", result.key?(:html)
assert_true "has changes array", result.key?(:changes) && result[:changes].is_a?(Array)
assert_true "reports changes", result[:changes].length > 0
assert_true "html is migrated", result[:html].include?("<column")

# --- validate ---

puts "validate:"

result = Inky.validate("<button>No href</button>")
assert_true "returns array", result.is_a?(Array)
rules = result.map { |d| d[:rule] }
assert_true "finds button href issue", rules.include?("button-no-href")

result = Inky.validate('<img src="test.jpg">')
rules = result.map { |d| d[:rule] }
assert_true "finds missing alt", rules.include?("missing-alt")

result = Inky.validate('<container><button href="#">OK</button></container>')
errors = result.select { |d| d[:severity] == "error" }
assert_true "valid template has no errors", errors.length == 0

# --- version ---

puts "version:"

v = Inky.version
assert_true "returns a string", v.is_a?(String)
assert_true "looks like semver", v.count(".") >= 2
assert_true "is 2.x", v.start_with?("2.")

# --- Summary ---

puts
if $failed == 0
  puts "All #{$passed} tests passed."
else
  puts "#{$passed} passed, #{$failed} failed."
  exit 1
end

#!/usr/bin/env bash
# CLI tests for inky build and validate commands.
# Usage: bash tests/test-cli.sh (run from repo root)

set -uo pipefail

INKY="./target/release/inky"
TMPDIR=$(mktemp -d)
PASSED=0
FAILED=0

cleanup() { rm -rf "$TMPDIR"; }
trap cleanup EXIT

assert_contains() {
  local label="$1" output="$2" expected="$3"
  if echo "$output" | grep -q "$expected"; then
    echo "  PASS  $label"
    PASSED=$((PASSED + 1))
  else
    echo "  FAIL  $label"
    echo "        expected output to contain: $expected"
    echo "        got: $output"
    FAILED=$((FAILED + 1))
  fi
}

assert_not_contains() {
  local label="$1" output="$2" unexpected="$3"
  if echo "$output" | grep -q "$unexpected"; then
    echo "  FAIL  $label"
    echo "        output should NOT contain: $unexpected"
    echo "        got: $output"
    FAILED=$((FAILED + 1))
  else
    echo "  PASS  $label"
    PASSED=$((PASSED + 1))
  fi
}

assert_file_exists() {
  local label="$1" filepath="$2"
  if [[ -f "$filepath" ]]; then
    echo "  PASS  $label"
    PASSED=$((PASSED + 1))
  else
    echo "  FAIL  $label"
    echo "        file does not exist: $filepath"
    FAILED=$((FAILED + 1))
  fi
}

assert_file_not_exists() {
  local label="$1" filepath="$2"
  if [[ ! -f "$filepath" ]]; then
    echo "  PASS  $label"
    PASSED=$((PASSED + 1))
  else
    echo "  FAIL  $label"
    echo "        file should not exist: $filepath"
    FAILED=$((FAILED + 1))
  fi
}

# --------------------------------------------------------------------------
echo ""
echo "=== Validate Tests ==="
# --------------------------------------------------------------------------

# File with multiple known issues
cat > "$TMPDIR/bad.html" <<'HTML'
<img src="logo.png">
<button>Click Me</button>
<p>No container here</p>
HTML

OUTPUT=$($INKY validate "$TMPDIR/bad.html" 2>&1)

assert_contains "missing-alt detected"       "$OUTPUT" "missing-alt"
assert_contains "button-no-href detected"    "$OUTPUT" "button-no-href"
assert_contains "missing-container detected" "$OUTPUT" "missing-container"
assert_contains "missing-preheader detected" "$OUTPUT" "missing-preheader"
assert_contains "img-no-width detected"      "$OUTPUT" "img-no-width"

# Clean file (as clean as possible)
cat > "$TMPDIR/clean.html" <<'HTML'
<container>
  <row>
    <columns>
      <spacer size="10"></spacer>
      <span class="preheader">Preview text</span>
      <p>Hello world</p>
    </columns>
  </row>
</container>
HTML

OUTPUT=$($INKY validate "$TMPDIR/clean.html" 2>&1)

assert_not_contains "clean file: no missing-alt"       "$OUTPUT" "missing-alt"
assert_not_contains "clean file: no button-no-href"    "$OUTPUT" "button-no-href"
assert_not_contains "clean file: no missing-container" "$OUTPUT" "missing-container"
assert_not_contains "clean file: no img-no-width"      "$OUTPUT" "img-no-width"

# --------------------------------------------------------------------------
echo ""
echo "=== .inky Extension Tests ==="
# --------------------------------------------------------------------------

# Single .inky file produces .html output in same directory
cat > "$TMPDIR/email.inky" <<'HTML'
<container><row><columns><p>Hello</p></columns></row></container>
HTML

$INKY build --no-inline-css "$TMPDIR/email.inky" > /dev/null 2>&1

assert_file_exists ".inky file produces .html output" "$TMPDIR/email.html"

# Verify the .html output contains transformed table markup
if [[ -f "$TMPDIR/email.html" ]]; then
  CONTENT=$(cat "$TMPDIR/email.html")
  assert_contains ".html output has table markup" "$CONTENT" "<table"
fi

# .html input goes to stdout (no file created)
cat > "$TMPDIR/stdin-test.html" <<'HTML'
<container><row><columns><p>Stdout</p></columns></row></container>
HTML

OUTPUT=$($INKY build --no-inline-css "$TMPDIR/stdin-test.html" 2>&1)
assert_contains ".html input goes to stdout" "$OUTPUT" "<table"

# Directory mode picks up .inky files
mkdir -p "$TMPDIR/dirtest" "$TMPDIR/dirout"
cat > "$TMPDIR/dirtest/one.inky" <<'HTML'
<container><row><columns><p>One</p></columns></row></container>
HTML
cat > "$TMPDIR/dirtest/two.inky" <<'HTML'
<container><row><columns><p>Two</p></columns></row></container>
HTML

$INKY build --no-inline-css "$TMPDIR/dirtest/" -o "$TMPDIR/dirout/" > /dev/null 2>&1

assert_file_exists "directory mode: one.html created" "$TMPDIR/dirout/one.html"
assert_file_exists "directory mode: two.html created" "$TMPDIR/dirout/two.html"

# Verify directory output contains transformed markup
if [[ -f "$TMPDIR/dirout/one.html" ]]; then
  CONTENT=$(cat "$TMPDIR/dirout/one.html")
  assert_contains "directory output has table markup" "$CONTENT" "<table"
fi

# --------------------------------------------------------------------------
echo ""
echo "=== Custom Component Tests ==="
# --------------------------------------------------------------------------

# Set up files for single-file component tests (components dir relative to input)
mkdir -p "$TMPDIR/comp-test/components"

cat > "$TMPDIR/comp-test/components/card.inky" <<'HTML'
<container>
<row>
  <column sm="12" lg="12">
    <callout>
      <h2>$title$</h2>
      <yield>
    </callout>
  </column>
</row>
</container>
HTML

cat > "$TMPDIR/comp-test/components/badge.inky" <<'HTML'
<span class="badge $color|blue$">$text|New$</span>
HTML

# Test: basic custom component with body
cat > "$TMPDIR/comp-test/basic.inky" <<'HTML'
<ink-card title="Hello">
  <p>Card content here</p>
</ink-card>
HTML

OUTPUT=$($INKY build --no-inline-css --no-framework-css "$TMPDIR/comp-test/basic.inky" -o /dev/stdout 2>/dev/null)
assert_contains "custom component: title replaced" "$OUTPUT" "Hello"
assert_contains "custom component: body injected" "$OUTPUT" "Card content here"
assert_contains "custom component: callout transformed" "$OUTPUT" "callout"

# Test: self-closing custom component
cat > "$TMPDIR/comp-test/selfclose.inky" <<'HTML'
<ink-badge text="Sale" color="red" />
HTML

OUTPUT=$($INKY build --no-inline-css --no-framework-css "$TMPDIR/comp-test/selfclose.inky" -o /dev/stdout 2>/dev/null)
assert_contains "self-closing component: text replaced" "$OUTPUT" "Sale"
assert_contains "self-closing component: color replaced" "$OUTPUT" "red"

# Test: custom component with defaults
cat > "$TMPDIR/comp-test/defaults.inky" <<'HTML'
<ink-badge />
HTML

OUTPUT=$($INKY build --no-inline-css --no-framework-css "$TMPDIR/comp-test/defaults.inky" -o /dev/stdout 2>/dev/null)
assert_contains "component defaults: text" "$OUTPUT" "New"
assert_contains "component defaults: color" "$OUTPUT" "blue"

# Test: nested custom components
cat > "$TMPDIR/comp-test/nested.inky" <<'HTML'
<ink-card title="Featured">
  <ink-badge text="Hot" color="red" />
  <p>Nested content</p>
</ink-card>
HTML

OUTPUT=$($INKY build --no-inline-css --no-framework-css "$TMPDIR/comp-test/nested.inky" -o /dev/stdout 2>/dev/null)
assert_contains "nested components: outer title" "$OUTPUT" "Featured"
assert_contains "nested components: inner badge" "$OUTPUT" "Hot"
assert_contains "nested components: body content" "$OUTPUT" "Nested content"

# Test: custom components with config directory
mkdir -p "$TMPDIR/comp-project/src" "$TMPDIR/comp-project/components" "$TMPDIR/comp-project/dist"

cp "$TMPDIR/comp-test/components/card.inky" "$TMPDIR/comp-project/components/"
cp "$TMPDIR/comp-test/basic.inky" "$TMPDIR/comp-project/src/"

cat > "$TMPDIR/comp-project/inky.config.json" <<'JSON'
{
  "src": "src",
  "dist": "dist",
  "components": "components"
}
JSON

$INKY build --no-inline-css --no-framework-css "$TMPDIR/comp-project" > /dev/null 2>&1
assert_file_exists "config components dir: output created" "$TMPDIR/comp-project/dist/basic.html"

if [[ -f "$TMPDIR/comp-project/dist/basic.html" ]]; then
  CONTENT=$(cat "$TMPDIR/comp-project/dist/basic.html")
  assert_contains "config components dir: output correct" "$CONTENT" "Hello"
fi

# --------------------------------------------------------------------------
echo ""
echo "=== Init Scaffold Tests ==="
# --------------------------------------------------------------------------

$INKY init "$TMPDIR/init-test" > /dev/null 2>&1
assert_file_exists "init: config created" "$TMPDIR/init-test/inky.config.json"
assert_file_exists "init: components dir created" "$TMPDIR/init-test/src/components/cta.inky"

if [[ -f "$TMPDIR/init-test/inky.config.json" ]]; then
  CONTENT=$(cat "$TMPDIR/init-test/inky.config.json")
  assert_contains "init: config has components" "$CONTENT" "components"
fi

# --------------------------------------------------------------------------
echo ""
echo "=== Results ==="
echo "$PASSED passed, $FAILED failed"
echo ""

if [[ $FAILED -gt 0 ]]; then
  exit 1
fi

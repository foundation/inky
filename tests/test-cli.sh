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
echo "=== Results ==="
echo "$PASSED passed, $FAILED failed"
echo ""

if [[ $FAILED -gt 0 ]]; then
  exit 1
fi

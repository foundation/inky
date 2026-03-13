"""
Tests for the Inky Python package.
Run: python3 test.py (after building libinky with `cargo build -p inky-ffi --release`)
"""

import sys
import os

# Add src to path for development
sys.path.insert(0, os.path.join(os.path.dirname(__file__), "src"))

from inky import transform, transform_inline, migrate, migrate_with_details, validate, version

passed = 0
failed = 0


def assert_true(name, condition, detail=""):
    global passed, failed
    if condition:
        passed += 1
    else:
        failed += 1
        print(f"  FAIL: {name}")
        if detail:
            print(f"        {detail}")


def assert_equal(name, actual, expected):
    global passed, failed
    if actual == expected:
        passed += 1
    else:
        failed += 1
        print(f"  FAIL: {name}")
        print(f"    expected: {expected}")
        print(f"    got:      {actual}")


# --- transform ---

print("transform:")

result = transform('<button href="#">Click</button>')
assert_true("button produces table", 'class="button"' in result)
assert_true("button has href", 'href="#"' in result)
assert_true("button has role=presentation", 'role="presentation"' in result)

result = transform("<container><row><column>Content</column></row></container>")
assert_true("full layout transforms", 'class="container"' in result)
assert_true("row transforms", 'class="row"' in result)
assert_true("column transforms", "columns" in result)

result = transform('<button href="#" size="small" color="alert">Click</button>')
assert_true("v2 size attribute becomes class", "small" in result)
assert_true("v2 color attribute becomes class", "alert" in result)

result = transform('<spacer height="20"></spacer>')
assert_true("spacer with v2 height", 'height="20"' in result)

result = transform("<divider></divider>")
assert_true("divider transforms", 'class="divider"' in result)

result = transform('<image src="hero.jpg" alt="Hero" width="600">')
assert_true("image transforms to img", "<img" in result)
assert_true("image has width", 'width="600"' in result)

result = transform("<outlook><p>MSO only</p></outlook>")
assert_true("outlook conditional", "<!--[if mso]>" in result)

result = transform("<not-outlook><p>Modern</p></not-outlook>")
assert_true("not-outlook conditional", "<!--[if !mso]><!-->" in result)

# --- transformInline ---

print("transformInline:")

result = transform_inline(
    '<html><head><style>.button { background: red; }</style></head>'
    '<body><button href="#">Click</button></body></html>'
)
assert_true("inlines CSS", "background" in result)
assert_true("transforms components", 'role="presentation"' in result)

# --- transform with columns ---

print("transform with columns:")

result = transform("<column>Content</column>", columns=16)
assert_true("custom column count", "small-16" in result or "large-16" in result)

# --- migrate ---

print("migrate:")

result = migrate('<columns large="6" small="12">Content</columns>')
assert_equal("columns to column", result, '<column lg="6" sm="12">Content</column>')

result = migrate("<h-line></h-line>")
assert_equal("h-line to divider", result, "<divider></divider>")

result = migrate('<spacer size="16"></spacer>')
assert_equal("spacer size to height", result, '<spacer height="16"></spacer>')

# --- migrate_with_details ---

print("migrate_with_details:")

result = migrate_with_details('<columns large="6">Content</columns>')
assert_true("returns dict", isinstance(result, dict))
assert_true("has html field", "html" in result)
assert_true("has changes list", "changes" in result and isinstance(result["changes"], list))
assert_true("reports changes", len(result["changes"]) > 0)
assert_true("html is migrated", "<column" in result["html"])

# --- validate ---

print("validate:")

result = validate("<button>No href</button>")
assert_true("returns list", isinstance(result, list))
rules = [d["rule"] for d in result]
assert_true("finds button href issue", "button-no-href" in rules)

result = validate('<img src="test.jpg">')
rules = [d["rule"] for d in result]
assert_true("finds missing alt", "missing-alt" in rules)

result = validate('<container><button href="#">OK</button></container>')
errors = [d for d in result if d["severity"] == "error"]
assert_true("valid template has no errors", len(errors) == 0)

# --- version ---

print("version:")

v = version()
assert_true("returns a string", isinstance(v, str))
assert_true("looks like semver", v.count(".") >= 2)
assert_true("is 2.x", v.startswith("2."))

# --- Summary ---

print()
if failed == 0:
    print(f"All {passed} tests passed.")
else:
    print(f"{passed} passed, {failed} failed.")
    sys.exit(1)

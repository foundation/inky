/**
 * Tests for the Inky Node.js package.
 * Run: node test.js (after building WASM with `npm run build:node`)
 */

const { transform, transformInline, migrate, migrateWithDetails, validate, version } = require("./index");

let passed = 0;
let failed = 0;

function assert(name, condition, detail) {
  if (condition) {
    passed++;
  } else {
    failed++;
    console.error(`  FAIL: ${name}`);
    if (detail) console.error(`        ${detail}`);
  }
}

function assertEqual(name, actual, expected) {
  if (actual === expected) {
    passed++;
  } else {
    failed++;
    console.error(`  FAIL: ${name}`);
    console.error(`    expected: ${expected}`);
    console.error(`    got:      ${actual}`);
  }
}

// --- transform ---

console.log("transform:");

{
  const result = transform('<button href="#">Click</button>');
  assert("button produces table", result.includes('class="button"'));
  assert("button has href", result.includes('href="#"'));
  assert("button has role=presentation", result.includes('role="presentation"'));
}

{
  const result = transform("<container><row><column>Content</column></row></container>");
  assert("full layout transforms", result.includes('class="container"'));
  assert("row transforms", result.includes('class="row"'));
  assert("column transforms", result.includes("columns"));
}

{
  const result = transform('<button href="#" size="small" color="alert">Click</button>');
  assert("v2 size attribute becomes class", result.includes("small"));
  assert("v2 color attribute becomes class", result.includes("alert"));
}

{
  const result = transform("<spacer height=\"20\"></spacer>");
  assert("spacer with v2 height", result.includes('height="20"'));
}

{
  const result = transform("<divider></divider>");
  assert("divider transforms", result.includes('class="divider"'));
}

{
  const result = transform('<image src="hero.jpg" alt="Hero" width="600">');
  assert("image transforms to img", result.includes("<img"));
  assert("image has width", result.includes('width="600"'));
}

{
  const result = transform("<outlook><p>MSO only</p></outlook>");
  assert("outlook conditional", result.includes("<!--[if mso]>"));
}

{
  const result = transform("<not-outlook><p>Modern</p></not-outlook>");
  assert("not-outlook conditional", result.includes("<!--[if !mso]><!-->"));
}

// --- transformInline ---

console.log("transformInline:");

{
  const result = transformInline('<html><head><style>.button { background: red; }</style></head><body><button href="#">Click</button></body></html>');
  assert("inlines CSS", result.includes("background"));
  assert("transforms components", result.includes('role="presentation"'));
}

// --- transform with options ---

console.log("transform with options:");

{
  const result = transform("<column>Content</column>", { columns: 16 });
  assert("custom column count", result.includes("small-16") || result.includes("large-16"));
}

// --- migrate ---

console.log("migrate:");

{
  const result = migrate('<columns large="6" small="12">Content</columns>');
  assertEqual("columns to column", result, '<column lg="6" sm="12">Content</column>');
}

{
  const result = migrate("<h-line></h-line>");
  assertEqual("h-line to divider", result, "<divider></divider>");
}

{
  const result = migrate('<spacer size="16"></spacer>');
  assertEqual("spacer size to height", result, '<spacer height="16"></spacer>');
}

// --- migrateWithDetails ---

console.log("migrateWithDetails:");

{
  const result = migrateWithDetails('<columns large="6">Content</columns>');
  assert("returns object", typeof result === "object");
  assert("has html field", typeof result.html === "string");
  assert("has changes array", Array.isArray(result.changes));
  assert("reports changes", result.changes.length > 0);
  assert("html is migrated", result.html.includes("<column"));
}

// --- validate ---

console.log("validate:");

{
  const result = validate('<button>No href</button>');
  assert("returns array", Array.isArray(result));
  assert("finds button href issue", result.some(d => d.rule === "button-no-href"));
}

{
  const result = validate('<img src="test.jpg">');
  assert("finds missing alt", result.some(d => d.rule === "missing-alt"));
}

{
  const result = validate('<container><button href="#">OK</button></container>');
  const errors = result.filter(d => d.severity === "error");
  assert("valid template has no errors", errors.length === 0);
}

// --- version ---

console.log("version:");

{
  const v = version();
  assert("returns a string", typeof v === "string");
  assert("looks like semver", /^\d+\.\d+\.\d+/.test(v));
  assert("is 2.x", v.startsWith("2."));
}

// --- Summary ---

console.log("");
if (failed === 0) {
  console.log(`All ${passed} tests passed.`);
} else {
  console.log(`${passed} passed, ${failed} failed.`);
  process.exit(1);
}

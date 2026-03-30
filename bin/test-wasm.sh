#!/bin/bash
# WASM build, test, and browser playground.
#
# Usage:
#   bin/test-wasm.sh          # run Node.js smoke tests
#   bin/test-wasm.sh --browse # build for web and open browser playground
#
# Requires: wasm-pack (cargo install wasm-pack), node

set -e
cd "$(dirname "$0")/.."

# Ensure cargo bin is in PATH
export PATH="$HOME/.cargo/bin:$PATH"

if [ "$1" = "--browse" ] || [ "$1" = "-b" ]; then
  echo "=== Building WASM package (web target) ==="
  wasm-pack build crates/inky-wasm --target web --out-dir ../../build/wasm-web 2>&1

  echo ""
  echo "=== Starting browser playground ==="

  # Create the playground HTML
  cat > build/wasm-web/index.html << 'HTMLEOF'
<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<title>Inky Playground</title>
<style>
  * { box-sizing: border-box; margin: 0; padding: 0; }
  body { font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif; background: #1a1a2e; color: #e0e0e0; height: 100vh; display: flex; flex-direction: column; }
  header { background: #16213e; padding: 12px 20px; display: flex; align-items: center; gap: 16px; border-bottom: 1px solid #2a2a4a; }
  header h1 { font-size: 18px; font-weight: 600; color: #fff; }
  header .version { font-size: 12px; color: #888; background: #2a2a4a; padding: 2px 8px; border-radius: 4px; }
  header .actions { margin-left: auto; display: flex; gap: 8px; }
  header button { background: #0f3460; color: #e0e0e0; border: 1px solid #2a2a4a; padding: 6px 14px; border-radius: 4px; cursor: pointer; font-size: 13px; }
  header button:hover { background: #1a4a80; }
  header button.primary { background: #e94560; border-color: #e94560; color: #fff; }
  header button.primary:hover { background: #ff6b81; }
  .panels { flex: 1; display: flex; overflow: hidden; }
  .panel { flex: 1; display: flex; flex-direction: column; min-width: 0; }
  .panel + .panel { border-left: 1px solid #2a2a4a; }
  .panel-header { background: #16213e; padding: 8px 14px; font-size: 12px; font-weight: 600; text-transform: uppercase; letter-spacing: 0.5px; color: #888; display: flex; align-items: center; gap: 8px; }
  .panel-header .tag { font-size: 10px; background: #2a2a4a; padding: 1px 6px; border-radius: 3px; text-transform: none; letter-spacing: 0; }
  textarea { flex: 1; background: #0f0f23; color: #e0e0e0; border: none; padding: 14px; font-family: "SF Mono", "Fira Code", monospace; font-size: 13px; line-height: 1.5; resize: none; outline: none; tab-size: 2; }
  textarea::placeholder { color: #444; }
  .output { flex: 1; overflow: auto; }
  .output pre { padding: 14px; font-family: "SF Mono", "Fira Code", monospace; font-size: 13px; line-height: 1.5; white-space: pre-wrap; word-wrap: break-word; }
  .output iframe { width: 100%; height: 100%; border: none; background: #fff; }
  .diagnostics { background: #1a1a2e; border-top: 1px solid #2a2a4a; max-height: 120px; overflow-y: auto; font-size: 12px; }
  .diagnostics:empty { display: none; }
  .diag { padding: 4px 14px; font-family: "SF Mono", "Fira Code", monospace; }
  .diag.warning { color: #f0c040; }
  .diag.error { color: #e94560; }
  .tabs { display: flex; gap: 0; }
  .tabs button { background: none; border: none; color: #666; padding: 8px 14px; font-size: 12px; cursor: pointer; border-bottom: 2px solid transparent; }
  .tabs button.active { color: #e0e0e0; border-bottom-color: #e94560; }
</style>
</head>
<body>
<header>
  <h1>Inky Playground</h1>
  <span class="version" id="version">loading...</span>
  <div class="actions">
    <button onclick="loadExample()">Load Example</button>
    <button class="primary" onclick="doTransform()">Transform</button>
  </div>
</header>
<div class="panels">
  <div class="panel">
    <div class="panel-header">Input <span class="tag">Inky HTML</span></div>
    <textarea id="input" placeholder="Type Inky markup here..." spellcheck="false"></textarea>
  </div>
  <div class="panel">
    <div class="panel-header">
      Output
      <div class="tabs">
        <button class="active" onclick="setTab('html', this)">HTML</button>
        <button onclick="setTab('preview', this)">Preview</button>
        <button onclick="setTab('migrate', this)">Migrate</button>
      </div>
    </div>
    <div class="output" id="output-html"><pre id="output-code"></pre></div>
    <div class="output" id="output-preview" style="display:none"><iframe id="preview-frame"></iframe></div>
    <div class="output" id="output-migrate" style="display:none"><pre id="migrate-code"></pre></div>
    <div class="diagnostics" id="diagnostics"></div>
  </div>
</div>

<script type="module">
import init, { transform, transform_with_config, validate, migrate, migrate_with_details, version } from './inky.js';

await init();

document.getElementById('version').textContent = 'v' + version();

const example = `<container>
  <row>
    <column sm="12" lg="8">
      <h1>Welcome to Inky</h1>
      <p>This is a live playground for the Inky email framework.</p>
      <button href="https://github.com/nicholasgasior/inky" class="large">Learn More</button>
    </column>
    <column sm="12" lg="4">
      <callout>
        <p>Tip: Edit the input and click Transform (or press Cmd+Enter).</p>
      </callout>
    </column>
  </row>
  <row>
    <column>
      <spacer height="16"></spacer>
      <divider></divider>
      <spacer height="16"></spacer>
      <center>
        <menu>
          <item href="#">Home</item>
          <item href="#">About</item>
          <item href="#">Contact</item>
        </menu>
      </center>
    </column>
  </row>
</container>`;

window.doTransform = function() {
  const input = document.getElementById('input').value;
  const result = transform(input);

  document.getElementById('output-code').textContent = result;

  // Update preview
  const frame = document.getElementById('preview-frame');
  frame.srcdoc = result;

  // Update migrate
  const migrateResult = migrate_with_details(input);
  const parsed = JSON.parse(migrateResult);
  if (parsed.changes.length > 0) {
    document.getElementById('migrate-code').textContent = parsed.html + '\n\n--- Changes ---\n' + parsed.changes.join('\n');
  } else {
    document.getElementById('migrate-code').textContent = '(no v1 syntax found)';
  }

  // Diagnostics
  const diags = JSON.parse(validate(input));
  const container = document.getElementById('diagnostics');
  container.innerHTML = '';
  for (const d of diags) {
    const el = document.createElement('div');
    el.className = 'diag ' + d.severity;
    el.textContent = `[${d.severity}] ${d.rule}: ${d.message}`;
    container.appendChild(el);
  }
};

window.loadExample = function() {
  document.getElementById('input').value = example;
  doTransform();
};

window.setTab = function(tab, btn) {
  document.querySelectorAll('.tabs button').forEach(b => b.classList.remove('active'));
  btn.classList.add('active');
  document.getElementById('output-html').style.display = tab === 'html' ? '' : 'none';
  document.getElementById('output-preview').style.display = tab === 'preview' ? '' : 'none';
  document.getElementById('output-migrate').style.display = tab === 'migrate' ? '' : 'none';
};

// Keyboard shortcut: Cmd+Enter or Ctrl+Enter to transform
document.getElementById('input').addEventListener('keydown', (e) => {
  if ((e.metaKey || e.ctrlKey) && e.key === 'Enter') {
    e.preventDefault();
    doTransform();
  }
});

// Load example on start
loadExample();
</script>
</body>
</html>
HTMLEOF

  echo "  Playground ready at http://localhost:8787"
  echo "  Press Ctrl+C to stop"
  echo ""

  # Use python's built-in HTTP server (available on macOS)
  cd build/wasm-web
  python3 -m http.server 8787
else
  echo "=== Building WASM package ==="
  wasm-pack build crates/inky-wasm --target nodejs --out-dir ../../build/wasm 2>&1

  echo ""
  echo "=== Running WASM smoke tests ==="
  node --eval "
const inky = require('./build/wasm');

let passed = 0;
let failed = 0;

function test(name, fn) {
  try {
    fn();
    console.log('  ✓ ' + name);
    passed++;
  } catch (e) {
    console.log('  ✗ ' + name);
    console.log('    ' + e.message);
    failed++;
  }
}

function assert(condition, msg) {
  if (!condition) throw new Error(msg || 'assertion failed');
}

// version
test('version() returns a string', () => {
  const v = inky.version();
  assert(typeof v === 'string' && v.length > 0, 'got: ' + v);
  console.log('    version: ' + v);
});

// transform
test('transform() converts inky to table markup', () => {
  const html = '<container><row><column>Hello</column></row></container>';
  const result = inky.transform(html);
  assert(result.includes('<table'), 'expected table markup, got: ' + result.substring(0, 100));
  assert(result.includes('Hello'), 'expected content preserved');
});

// transform_with_config
test('transform_with_config() respects column count', () => {
  const html = '<container><row><column lg=\"6\">Half</column><column lg=\"6\">Half</column></row></container>';
  const result = inky.transform_with_config(html, 12);
  assert(result.includes('<table'), 'expected table markup');
});

// validate
test('validate() returns JSON diagnostics', () => {
  const html = '<row><column>No container</column></row>';
  const result = JSON.parse(inky.validate(html));
  assert(Array.isArray(result), 'expected array');
  assert(result.length > 0, 'expected diagnostics for missing container');
  assert(result[0].severity, 'expected severity field');
  assert(result[0].rule, 'expected rule field');
  console.log('    found ' + result.length + ' diagnostic(s)');
});

// validate_with_config
test('validate_with_config() accepts column count', () => {
  const html = '<container><row><column>OK</column></row></container>';
  const result = JSON.parse(inky.validate_with_config(html, 6));
  assert(Array.isArray(result), 'expected array');
});

// migrate
test('migrate() converts v1 to v2 syntax', () => {
  const html = '<columns>Content</columns>';
  const result = inky.migrate(html);
  assert(result.includes('<column>'), 'expected <column>, got: ' + result);
  assert(!result.includes('<columns>'), 'should not contain <columns>');
});

// migrate_with_details
test('migrate_with_details() returns JSON with changes', () => {
  const html = '<columns large=\"6\">Content</columns>';
  const result = JSON.parse(inky.migrate_with_details(html));
  assert(result.html, 'expected html field');
  assert(Array.isArray(result.changes), 'expected changes array');
  assert(result.changes.length > 0, 'expected at least one change');
  console.log('    changes: ' + result.changes.join(', '));
});

// summary
console.log('');
console.log(passed + ' passed, ' + failed + ' failed');
if (failed > 0) process.exit(1);
"
fi

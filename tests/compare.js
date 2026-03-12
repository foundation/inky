#!/usr/bin/env node
// Compare expected output with Rust Inky output for all templates.
// Usage: node tests/compare.js
//
// Expected output is pre-generated in tests/expected/.
// Known differences (attribute order, void tag closing, entities) are normalized.

const fs = require('fs');
const path = require('path');
const { execSync } = require('child_process');

const templatesDir = path.join(__dirname, 'templates');
const expectedDir = path.join(templatesDir, 'expected');
const files = fs.readdirSync(templatesDir).filter(f => f.endsWith('.html'));

let passed = 0;
let failed = 0;
let diffs = [];

// Normalize HTML for comparison
function normalize(html) {
  let s = html;
  // Collapse all whitespace to single spaces
  s = s.replace(/\s+/g, ' ');
  // Remove closing tags for void elements (html5ever adds these)
  s = s.replace(/<\/(img|br|hr|input|meta|link)>/gi, '');
  // Normalize HTML entities (html5ever decodes/re-encodes entities differently)
  s = s.replace(/&#x2014;/g, '\u2014');
  s = s.replace(/&#xA0;/g, '&nbsp;');
  s = s.replace(/&amp;/g, '&');
  // Normalize bare attributes: alt → alt=""
  s = s.replace(/<([a-z][a-z0-9-]*)((?:\s+[^\s>=/]+(?:=(?:"[^"]*"|'[^']*'|[^\s>]*))?)*)\s*>/gi,
    (match, tag, attrsStr) => {
      if (!attrsStr.trim()) return match;
      // Parse individual attributes
      const attrRegex = /([^\s>=/]+)(?:=(?:"([^"]*)"|'([^']*)'|([^\s>]*)))?/g;
      let attrs = [];
      let m;
      while ((m = attrRegex.exec(attrsStr)) !== null) {
        const name = m[1];
        const value = m[2] !== undefined ? m[2] : (m[3] !== undefined ? m[3] : (m[4] !== undefined ? m[4] : ''));
        attrs.push(`${name}="${value}"`);
      }
      attrs.sort();
      return `<${tag} ${attrs.join(' ')}>`;
    }
  );
  return s.trim();
}

for (const file of files) {
  // Read pre-generated expected output
  const expectedPath = path.join(expectedDir, file);
  if (!fs.existsSync(expectedPath)) {
    console.log(`  ? ${file} (no expected output)`);
    failed++;
    continue;
  }
  const expectedHtml = normalize(fs.readFileSync(expectedPath, 'utf8'));

  // Rust Inky transform
  let rustHtml;
  try {
    rustHtml = normalize(
      execSync(`./target/release/inky build --no-inline-css "${path.join(templatesDir, file)}"`, {
        encoding: 'utf8',
        stdio: ['pipe', 'pipe', 'pipe']
      })
    );
  } catch (e) {
    rustHtml = `ERROR: ${e.message}`;
  }

  if (expectedHtml === rustHtml) {
    console.log(`  \u2713 ${file}`);
    passed++;
  } else {
    console.log(`  \u2717 ${file}`);
    diffs.push({ file, expectedHtml, rustHtml });
    failed++;
  }
}

console.log(`\n${passed} passed, ${failed} failed out of ${files.length} templates\n`);

// Write diffs for inspection
if (diffs.length > 0) {
  const diffDir = '/tmp/inky-diffs';
  fs.mkdirSync(diffDir, { recursive: true });

  for (const d of diffs.slice(0, 5)) {
    const base = d.file.replace('.html', '');
    fs.writeFileSync(`${diffDir}/${base}-expected.html`, d.expectedHtml);
    fs.writeFileSync(`${diffDir}/${base}-rust.html`, d.rustHtml);
    console.log(`  ${d.file}:`);
    console.log(`    Expected: ${diffDir}/${base}-expected.html`);
    console.log(`    Rust:     ${diffDir}/${base}-rust.html`);

    // Show a short excerpt of the difference
    const expWords = d.expectedHtml.split(' ');
    const rustWords = d.rustHtml.split(' ');
    for (let i = 0; i < Math.min(expWords.length, rustWords.length); i++) {
      if (expWords[i] !== rustWords[i]) {
        const start = Math.max(0, i - 2);
        const end = Math.min(expWords.length, i + 3);
        console.log(`    First diff at word ${i}:`);
        console.log(`      Expected: ...${expWords.slice(start, end).join(' ')}...`);
        console.log(`      Rust:     ...${rustWords.slice(start, end).join(' ')}...`);
        break;
      }
    }
    console.log('');
  }
}

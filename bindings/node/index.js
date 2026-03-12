/**
 * Inky — Transform email templates into email-safe HTML
 *
 * This is a thin wrapper around the Rust WASM module.
 * Build the WASM first: npm run build:node
 */

let wasm;

try {
  wasm = require("./inky_wasm");
} catch (e) {
  throw new Error(
    "Inky WASM module not found. Run `npm run build:node` first.\n" + e.message
  );
}

/**
 * Transform Inky HTML into email-safe table markup.
 * @param {string} html - Inky template HTML
 * @param {object} [options] - Options
 * @param {number} [options.columns=12] - Number of grid columns
 * @returns {string} Transformed HTML
 */
function transform(html, options = {}) {
  const columns = options.columns || 12;
  if (columns !== 12) {
    return wasm.transform_with_config(html, columns);
  }
  return wasm.transform(html);
}

/**
 * Migrate v1 Inky syntax to v2.
 * @param {string} html - v1 Inky template HTML
 * @returns {string} Migrated v2 HTML
 */
function migrate(html) {
  return wasm.migrate(html);
}

/**
 * Migrate v1 syntax and return detailed results.
 * @param {string} html - v1 Inky template HTML
 * @returns {{ html: string, changes: string[] }} Migration result with list of changes
 */
function migrateWithDetails(html) {
  return JSON.parse(wasm.migrate_with_details(html));
}

/**
 * Validate an Inky template.
 * @param {string} html - Inky template HTML
 * @param {object} [options] - Options
 * @param {number} [options.columns=12] - Number of grid columns
 * @returns {Array<{ severity: string, rule: string, message: string }>} Diagnostics
 */
function validate(html, options = {}) {
  const columns = options.columns || 12;
  if (columns !== 12) {
    return JSON.parse(wasm.validate_with_config(html, columns));
  }
  return JSON.parse(wasm.validate(html));
}

/**
 * Get the Inky version.
 * @returns {string}
 */
function version() {
  return wasm.version();
}

module.exports = { transform, migrate, migrateWithDetails, validate, version };

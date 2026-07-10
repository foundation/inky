#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

/**
 * Transform Inky HTML to email-safe HTML.
 * Returns null if `input` is null or an internal error occurs.
 * Caller must free the returned string with inky_free().
 *
 * # Safety
 * `input` must be null or a valid null-terminated C string.
 */
char *inky_transform(const char *input);

/**
 * Transform with custom column count.
 * Returns null if `input` is null or an internal error occurs.
 * Caller must free the returned string with inky_free().
 *
 * # Safety
 * `input` must be null or a valid null-terminated C string.
 */
char *inky_transform_with_columns(const char *input, uint32_t column_count);

/**
 * Transform Inky HTML and inline CSS from `<style>` blocks.
 * Returns the result HTML, or the original transform output if inlining fails.
 * Returns null if `input` is null or an internal error occurs.
 * Caller must free the returned string with inky_free().
 *
 * # Safety
 * `input` must be null or a valid null-terminated C string.
 */
char *inky_transform_inline(const char *input);

/**
 * Transform Inky HTML with MiniJinja data merge, then inline CSS.
 *
 * `data_json` must be a valid JSON C string with merge variables.
 * Missing keys render as empty strings (lenient mode).
 * Returns null if any argument is null or an internal error occurs.
 * Caller must free the returned string with inky_free().
 *
 * # Safety
 * `input` and `data_json` must each be null or a valid null-terminated C string.
 */
char *inky_transform_with_data(const char *input, const char *data_json);

/**
 * Transform using hybrid output mode (div + MSO ghost tables).
 * Returns null if `input` is null or an internal error occurs.
 * Caller must free the returned string with inky_free().
 *
 * # Safety
 * `input` must be null or a valid null-terminated C string.
 */
char *inky_transform_hybrid(const char *input);

/**
 * Migrate v1 Inky syntax to v2.
 * Returns the migrated HTML string.
 * Returns null if `input` is null or an internal error occurs.
 * Caller must free the returned string with inky_free().
 *
 * # Safety
 * `input` must be null or a valid null-terminated C string.
 */
char *inky_migrate(const char *input);

/**
 * Migrate v1 syntax and return a JSON string with `html` and `changes` fields.
 * Returns null if `input` is null or an internal error occurs.
 * Caller must free the returned string with inky_free().
 *
 * # Safety
 * `input` must be null or a valid null-terminated C string.
 */
char *inky_migrate_with_details(const char *input);

/**
 * Validate an Inky template and return diagnostics as a JSON array.
 * Each entry has `severity`, `rule`, and `message` fields.
 * Returns null if `input` is null or an internal error occurs.
 * Caller must free the returned string with inky_free().
 *
 * # Safety
 * `input` must be null or a valid null-terminated C string.
 */
char *inky_validate(const char *input);

/**
 * Convert HTML to plain text for multipart email.
 * Returns null if `input` is null or an internal error occurs.
 * Caller must free the returned string with inky_free().
 *
 * # Safety
 * `input` must be null or a valid null-terminated C string.
 */
char *inky_to_plain_text(const char *input);

/**
 * Get the Inky version string.
 * Caller must free the returned string with inky_free().
 */
char *inky_version(void);

/**
 * Free a string returned by any inky_* function.
 *
 * # Safety
 * `ptr` must be a pointer returned by one of the inky_* functions, or null.
 */
void inky_free(char *ptr);

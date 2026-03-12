#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

/**
 * Transform Inky HTML to email-safe HTML.
 * Caller must free the returned string with inky_free().
 */
char *inky_transform(const char *input);

/**
 * Transform with custom column count.
 * Caller must free the returned string with inky_free().
 */
char *inky_transform_with_columns(const char *input, uint32_t column_count);

/**
 * Transform Inky HTML and inline CSS from `<style>` blocks.
 * Returns the result HTML, or the original transform output if inlining fails.
 * Caller must free the returned string with inky_free().
 */
char *inky_transform_inline(const char *input);

/**
 * Migrate v1 Inky syntax to v2.
 * Returns the migrated HTML string.
 * Caller must free the returned string with inky_free().
 */
char *inky_migrate(const char *input);

/**
 * Migrate v1 syntax and return a JSON string with `html` and `changes` fields.
 * Caller must free the returned string with inky_free().
 */
char *inky_migrate_with_details(const char *input);

/**
 * Validate an Inky template and return diagnostics as a JSON array.
 * Each entry has `severity`, `rule`, and `message` fields.
 * Caller must free the returned string with inky_free().
 */
char *inky_validate(const char *input);

/**
 * Get the Inky version string.
 * Caller must free the returned string with inky_free().
 */
char *inky_version(void);

/**
 * Free a string returned by any inky_* function.
 */
void inky_free(char *ptr);

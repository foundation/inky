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
 * Free a string returned by inky_transform or inky_transform_with_columns.
 */
void inky_free(char *ptr);

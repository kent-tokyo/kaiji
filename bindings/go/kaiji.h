#ifndef KAIJI_H
#define KAIJI_H

#ifdef __cplusplus
extern "C" {
#endif

/**
 * Normalize a NUL-terminated UTF-8 CJK string.
 * Returns a new heap-allocated string; the caller must free it with kaiji_free_string().
 * Returns NULL on error or NULL input.
 */
char *kaiji_normalize(const char *input);

/**
 * Return 1 if a and b are equivalent after CJK normalization, 0 if not, -1 on error.
 */
int kaiji_matches(const char *a, const char *b);

/**
 * Return the Jaro-Winkler similarity score in [0.0, 1.0] between a and b
 * after CJK normalization. Returns -1.0 on error.
 */
float kaiji_similarity(const char *a, const char *b);

/**
 * Free a string returned by kaiji_normalize().
 */
void kaiji_free_string(char *s);

#ifdef __cplusplus
}
#endif

#endif /* KAIJI_H */

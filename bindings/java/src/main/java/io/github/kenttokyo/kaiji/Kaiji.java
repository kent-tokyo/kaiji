package io.github.kenttokyo.kaiji;

/**
 * kaiji — CJK fuzzy match and normalization engine.
 *
 * <p>Before using this class, build the native library:
 * <pre>
 *   cargo build --release --manifest-path crates/kaiji-java/Cargo.toml
 * </pre>
 * Then load it in your application:
 * <pre>
 *   System.load("/path/to/libkaiji_java.dylib");  // macOS
 *   System.load("/path/to/libkaiji_java.so");     // Linux
 * </pre>
 */
public class Kaiji {
    /**
     * Normalize a CJK string. Folds variant characters (e.g., 齋→斉) and strips IVS selectors.
     *
     * @param input the string to normalize
     * @return the normalized string, or null on error
     */
    public static native String normalize(String input);

    /**
     * Return true if {@code a} and {@code b} are equivalent after CJK normalization.
     *
     * @param a first string
     * @param b second string
     * @return true if normalized forms are identical
     */
    public static native boolean matches(String a, String b);

    /**
     * Compute Jaro-Winkler similarity score (0.0–1.0) between two strings
     * after CJK normalization.
     *
     * @param a first string
     * @param b second string
     * @return similarity score in [0.0, 1.0], or -1.0 on error
     */
    public static native float similarityScore(String a, String b);
}

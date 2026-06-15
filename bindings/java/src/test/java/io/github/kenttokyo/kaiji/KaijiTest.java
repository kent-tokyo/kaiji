package io.github.kenttokyo.kaiji;

import org.junit.jupiter.api.Test;
import static org.junit.jupiter.api.Assertions.*;

/**
 * Integration tests for Kaiji JNI bindings.
 *
 * Requires the native library to be built and loaded before running:
 *   cargo build --release --manifest-path crates/kaiji-java/Cargo.toml
 *   System.load(System.getProperty("kaiji.lib.path"));
 */
public class KaijiTest {

    @Test
    void testNormalize() {
        String result = Kaiji.normalize("齋藤");
        assertEquals("斉藤", result);
    }

    @Test
    void testMatchesVariants() {
        assertTrue(Kaiji.matches("斎藤", "齋藤"));
        assertTrue(Kaiji.matches("渡辺", "渡邊"));
    }

    @Test
    void testMatchesDifferent() {
        assertFalse(Kaiji.matches("斎藤", "佐藤"));
    }

    @Test
    void testSimilarityScore() {
        float score = Kaiji.similarityScore("斎藤", "齋藤");
        assertEquals(1.0f, score, 0.001f);
    }
}

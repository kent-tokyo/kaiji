# kaiji Java Bindings

Java/JNI bindings for [kaiji](https://github.com/kent-tokyo/kaiji) — a high-performance CJK fuzzy search and text normalization engine written in Rust.

## Building the Native Library

First, build the Rust shared library:

```bash
cargo build --release --manifest-path crates/kaiji-java/Cargo.toml
```

This produces:
- `target/release/libkaiji_java.dylib` — macOS
- `target/release/libkaiji_java.so` — Linux
- `target/release/kaiji_java.dll` — Windows

## Usage

Copy the native library to a known path, then load it in your Java application before calling any `Kaiji` methods:

```java
import io.github.kenttokyo.kaiji.Kaiji;

System.load("/path/to/libkaiji_java.dylib"); // or .so / .dll

String normalized = Kaiji.normalize("齋藤");          // "斉藤"
boolean matched   = Kaiji.matches("斎藤", "齋藤");    // true
float   score     = Kaiji.similarityScore("斎藤", "齋藤"); // 1.0f
```

## Gradle Setup (once published to Maven Central)

```kotlin
dependencies {
    implementation("io.github.kenttokyo:kaiji-java:0.1.0")
}
```

You will still need to bundle and load the native library for your target platform.

## API Reference

| Method | Description |
|--------|-------------|
| `String normalize(String input)` | Folds CJK variant characters (e.g., 齋→斉) and strips IVS selectors. Returns `null` on error. |
| `boolean matches(String a, String b)` | Returns `true` if `a` and `b` are equivalent after CJK normalization. |
| `float similarityScore(String a, String b)` | Returns Jaro-Winkler similarity in [0.0, 1.0] after CJK normalization. Returns `-1.0` on error. |

## Running Tests

```bash
cargo build --release --manifest-path crates/kaiji-java/Cargo.toml
cd bindings/java
./gradlew test -Dkaiji.lib.path=/path/to/libkaiji_java.dylib
```

## Thread Safety

All methods are thread-safe. The Rust core initializes the variant character map once via `OnceLock` and shares it immutably across threads thereafter.

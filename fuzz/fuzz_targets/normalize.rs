#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let Ok(s) = std::str::from_utf8(data) else { return; };
    let cfg = kaiji::NormalizerConfig::default();
    let _ = kaiji::normalize(s, &cfg);
    let _ = kaiji::matches(s, s, &cfg);
    let _ = kaiji::similarity_score(s, s, &cfg);
    let n = kaiji::Normalizer::builder().build();
    let _ = n.normalize(s);
    let _ = n.matches(s, s);
});

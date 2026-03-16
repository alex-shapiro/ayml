#![no_main]

use libfuzzer_sys::fuzz_target;

// Deserialize arbitrary bytes — must never panic.
fuzz_target!(|data: &[u8]| {
    _ = ayml_serde::from_slice::<ayml_serde::Value>(data);
});

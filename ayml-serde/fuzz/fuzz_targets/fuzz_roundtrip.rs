#![no_main]

use libfuzzer_sys::fuzz_target;
use ayml_serde::Value;

// Parse arbitrary bytes, and if successful, roundtrip through
// serialize → deserialize and assert equality.
fuzz_target!(|data: &[u8]| {
    let Ok(value) = ayml_serde::from_slice::<Value>(data) else {
        return;
    };

    let serialized = ayml_serde::to_string(&value)
        .expect("serializing a successfully-parsed Value should not fail");

    let roundtripped: Value = ayml_serde::from_str(&serialized)
        .expect("deserializing our own serialized output should not fail");

    assert_eq!(value, roundtripped, "roundtrip mismatch");
});

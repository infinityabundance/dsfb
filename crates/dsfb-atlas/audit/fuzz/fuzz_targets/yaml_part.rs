#![no_main]
use libfuzzer_sys::fuzz_target;

// We deserialise into serde_yaml::Value rather than dsfb_atlas::schema::Part
// to avoid coupling the fuzz target to the (intentionally) private schema
// types; a future revision can also import the schema crate and parse into
// the typed Part directly.
fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        let _: Result<serde_yaml::Value, _> = serde_yaml::from_str(s);
    }
});

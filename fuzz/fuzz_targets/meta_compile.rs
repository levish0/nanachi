#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    nanachi_fuzz::exercise_bytes(data);
});

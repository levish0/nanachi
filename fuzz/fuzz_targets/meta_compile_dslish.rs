#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let input = nanachi_fuzz::project_to_dslish(data);
    nanachi_fuzz::exercise_utf8(&input);
});

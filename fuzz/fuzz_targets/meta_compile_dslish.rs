#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let input = faputa_fuzz::project_to_dslish(data);
    faputa_fuzz::exercise_utf8(&input);
});

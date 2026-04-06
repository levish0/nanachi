use faputa_meta::{compile, parser, validator};

const DSLISH_ALPHABET: &[u8] =
    b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789_ \n\t{}()|&!?+=,<>:*./'\"";

pub fn exercise_utf8(input: &str) {
    if let Ok(grammar) = parser::parse(input) {
        let _ = validator::validate(&grammar);
    }

    let _ = compile(input);
}

pub fn project_to_dslish(data: &[u8]) -> String {
    data.iter()
        .take(4096)
        .map(|byte| DSLISH_ALPHABET[*byte as usize % DSLISH_ALPHABET.len()] as char)
        .collect()
}

pub fn exercise_bytes(data: &[u8]) {
    if let Ok(input) = std::str::from_utf8(data) {
        exercise_utf8(input);
    }

    let projected = project_to_dslish(data);
    exercise_utf8(&projected);
}

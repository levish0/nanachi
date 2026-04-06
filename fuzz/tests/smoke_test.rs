use faputa_fuzz::{exercise_bytes, exercise_utf8, project_to_dslish};

fn read_corpus_dir(name: &str) -> Vec<Vec<u8>> {
    let path = format!("{}/corpus/{name}", env!("CARGO_MANIFEST_DIR"));
    let mut payloads = Vec::new();

    for entry in std::fs::read_dir(&path).unwrap_or_else(|e| panic!("{path}: {e}")) {
        let entry = entry.unwrap_or_else(|e| panic!("{path}: {e}"));
        let bytes = std::fs::read(entry.path()).unwrap_or_else(|e| panic!("{path}: {e}"));
        payloads.push(bytes);
    }

    assert!(!payloads.is_empty(), "{path} should not be empty");
    payloads
}

fn next_u64(state: &mut u64) -> u64 {
    *state = state
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407);
    *state
}

#[test]
fn fuzz_corpus_inputs_are_exercised() {
    for corpus_name in ["meta_compile", "meta_compile_dslish"] {
        for payload in read_corpus_dir(corpus_name) {
            exercise_bytes(&payload);
        }
    }
}

#[test]
fn deterministic_random_payloads_are_exercised() {
    let mut state = 0xC0FFEE_u64;

    for _ in 0..256 {
        let len = (next_u64(&mut state) % 512) as usize;
        let mut payload = Vec::with_capacity(len);
        for _ in 0..len {
            payload.push((next_u64(&mut state) & 0xFF) as u8);
        }
        exercise_bytes(&payload);
    }
}

#[test]
fn projected_random_payloads_are_utf8_and_exercised() {
    let mut state = 0xDEADBEEF_u64;

    for _ in 0..128 {
        let len = (next_u64(&mut state) % 256) as usize;
        let mut payload = Vec::with_capacity(len);
        for _ in 0..len {
            payload.push((next_u64(&mut state) & 0xFF) as u8);
        }

        let projected = project_to_dslish(&payload);
        exercise_utf8(&projected);
    }
}

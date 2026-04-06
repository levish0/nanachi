/// Parse a .faputa file and print its AST.
///
/// ```sh
/// cargo run -p faputa_meta --example parse -- examples/simple.faputa
/// cargo run -p faputa_meta --example parse -- examples/markdown_bold.faputa
/// ```

fn main() {
    let path = std::env::args().nth(1).unwrap_or_else(|| {
        eprintln!("Usage: parse <file.faputa>");
        std::process::exit(1);
    });

    let source = std::fs::read_to_string(&path).unwrap_or_else(|e| {
        eprintln!("Error reading {path}: {e}");
        std::process::exit(1);
    });

    match faputa_meta::compile(&source) {
        Ok(grammar) => {
            println!("=== Parsed {path} ===\n");
            for item in &grammar.items {
                println!("{item:#?}\n");
            }
        }
        Err(e) => {
            eprintln!("Error: {e:?}");
            std::process::exit(1);
        }
    }
}

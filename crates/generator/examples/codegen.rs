/// Generate winnow parser code from a .faputa file.
///
/// ```sh
/// cargo run -p faputa_generator --example codegen -- examples/simple.faputa
/// cargo run -p faputa_generator --example codegen -- examples/markdown_bold.faputa
/// ```
///
/// Set `RUST_LOG=debug` to see tracing output from the compilation pipeline.

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_writer(std::io::stderr)
        .init();

    let path = std::env::args().nth(1).unwrap_or_else(|| {
        eprintln!("Usage: codegen <file.faputa>");
        std::process::exit(1);
    });

    let source = std::fs::read_to_string(&path).unwrap_or_else(|e| {
        eprintln!("Error reading {path}: {e}");
        std::process::exit(1);
    });

    let grammar = faputa_meta::compile(&source).unwrap_or_else(|e| {
        eprintln!("Error: {e:?}");
        std::process::exit(1);
    });

    let code = faputa_generator::generate(&grammar);

    // Pretty-print if possible, fall back to raw token stream
    match syn::parse2::<syn::File>(code.clone()) {
        Ok(file) => print!("{}", prettyplease::unparse(&file)),
        Err(_) => print!("{code}"),
    }
}

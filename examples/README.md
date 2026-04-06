# faputa examples

Example `.faputa` grammar files and how to run them.

## Files

| File                    | Description                                           |
|-------------------------|-------------------------------------------------------|
| `simple.faputa`        | Basic grammar: identifiers from letters and digits    |
| `markdown_bold.faputa` | Stateful parsing: bold markers with re-entrance guard |
| `nested_braces.faputa` | Counter-based nesting with depth limit                |

## Parse a grammar (prints AST)

```sh
cargo run -p faputa_meta --example parse -- examples/simple.faputa
cargo run -p faputa_meta --example parse -- examples/markdown_bold.faputa
cargo run -p faputa_meta --example parse -- examples/nested_braces.faputa
```

## Generate winnow code

```sh
cargo run -p faputa_generator --example codegen -- examples/simple.faputa
cargo run -p faputa_generator --example codegen -- examples/markdown_bold.faputa
cargo run -p faputa_generator --example codegen -- examples/nested_braces.faputa
```

## Run all tests

```sh
cargo test --workspace
```

# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.4] - 2026-04-06

### Added

- **`@` error label syntax** — opt-in custom error messages at two levels
  - Rule-level: `value = @ "json value" { ... }` — sets the Label context for the entire rule (error shows "invalid json value" instead of "invalid value")
  - Expression-level: `number @ "a number" | ident @ "an identifier"` — attaches Expected context to individual expressions or choice branches
  - `@` binds tighter than `|` but after postfix operators (`+`, `*`, `?`, `{n,m}`), so `digit+ @ "digits" | alpha` works as expected
  - Full pipeline support: lexer (`At` token) → parser → AST (`Expr::Labeled`, `RuleDef.error_label`) → validator → IR (`IrExpr::Labeled`, `IrRule.error_label`) → optimizer (all 8+ passes) → generator

- **Tracing instrumentation** across the compilation pipeline
  - `compile()`, `parse()`, `validate()`, `lower()`, `optimize()`, and `generate_module_inner()` are instrumented with `#[tracing::instrument]`
  - Optimizer logs each phase with structured fields (rule count, inlined count, entry points)
  - Set `RUST_LOG=debug` on any example binary to see the full pipeline trace on stderr

- **`error_labels` example** (`examples/error_labels`)
  - Demonstrates both rule-level and expression-level `@` labels
  - Shows how custom error messages appear for different failure modes

- **`tracing-subscriber`** added to all example binaries (`parse_json`, `parse_demo`, `error_labels`)
  - Initialized with `EnvFilter` writing to stderr, so parser output stays on stdout

### Changed

- **Choice branches no longer emit automatic `.context(Expected(...))`**
  - Previously every choice branch got a heuristic description via `describe_expr()` — this added `.context()` calls on hot paths with often-unhelpful messages
  - Now only explicit `@ "label"` annotations add Expected context
  - Rule-level `Label` context is always present (using `error_label` if set, else rule name)
  - Deleted `describe_expr()` and `describe_ranges()` helper functions from generator

- **Default error strategy is now lean**: rule Label + opt-in Expected via `@`
  - This addresses the 7x `.context()` overhead identified in 0.1.2 profiling while preserving user-facing error quality where authors choose to add it

### Fixed

- **Error position accuracy in sequences** — `track_pos()` is now interleaved between sequence elements in generated code
  - Previously `track_pos()` was only called at rule entry, so errors within a sequence (e.g., `ident "=" value` failing at `"="`) reported the rule start position instead of the actual failure point
  - `x+42` against `assign = { ident "=" value }` now correctly reports `1:2` (at `+`) instead of `1:1`
  - Large sequences (>11 elements after interleaving) fall back to explicit sequential parsing to stay within winnow's 21-element tuple limit

### Internal

- Test fixtures updated: `@` was previously used as an "unexpected character" in test cases; replaced with `$` since `@` is now valid syntax
- `tracing` dependency added to `faputa_meta` and `faputa_generator`
- `tracing-subscriber` added to workspace dependencies

## [0.1.3] - 2026-04-06

### Added

- **IR optimization pipeline** (`faputa_meta::ir::optimize`)
  - New `single_char_to_charset` pass: converts single-character `Literal("x")` → `CharSet` inside `Choice` branches, enabling downstream merging (e.g. `" " | "\t" | "\n"` → single `CharSet`)
  - New `recognize_take_while` pass: fuses `Repeat { CharSet(ranges), min, max }` patterns into `TakeWhile` — maps directly to winnow's SIMD-accelerated `take_while()`
  - New `compute_ref_counts` pass: call-graph analysis to distinguish entry-point rules (`ref_count == 0`) from internal rules (`ref_count > 0`)
  - Extended `is_trivial` to include `TakeWhile` variant for more aggressive inlining
  - Reordered pipeline with two normalization phases: pre-inline (single_char → flatten → merge → fuse) and post-inline (flatten → merge → fuse → recognize_take_while) to maximize optimization opportunities

- **New IR node: `TakeWhile`** (`faputa_meta::ir::expr`)
  - Represents fused character-class repeats: `(' ' | '\t' | '\n' | '\r')*` → `TakeWhile { ranges, min: 0 }`
  - Enables winnow `take_while(0.., (' ', '\t', '\n', '\r'))` codegen with SIMD/memchr support

- **`ref_count` field on `IrRule`** (`faputa_meta::ir::program`)
  - Tracks how many other rules reference each rule
  - Used by generator to apply different wrapping strategies for entry vs internal rules

### Changed

- **Generator rewritten to IR-based codegen** (`faputa_generator`)
  - `expr.rs`: Fully rewritten — generates winnow code from `IrExpr` instead of AST `Expr`
    - `CharSet` → `one_of(tuple)` for ≤10 ranges, closure fallback for >10
    - `TakeWhile` → `take_while(range, set)` with tuple or closure
    - Boundary expressions generate lightweight closures instead of `trace()`-wrapped blocks
    - Stateful expressions (`WithFlag`, `WithCounter`, `When`, `DepthLimit`) generate minimal closures without `trace()` wrappers
  - `rules.rs`: Fully rewritten — uses `IrRule` with entry/internal distinction
    - Entry points (`ref_count == 0`): full `trace()` + `.context(Label)` + `track_pos()`
    - Internal rules (`ref_count > 0`): minimal wrapper, just `.take()` for return type
  - `statement.rs`: Updated signature — accepts `(&[GuardCondition], &[String])` instead of `&[Statement]`
  - `lib.rs`: Pipeline now runs `lower → optimize → IR-based codegen`
  - `state.rs`: Uses `IrProgram.state_decls` instead of iterating `Grammar.items`

- **Per-terminal `.context(Expected(...))` removed** from generated code
  - Previously every `literal()` and `one_of()` had individual error context annotations
  - Now only entry-point rules have `.context(Label("rule_name"))` — significant performance improvement
  - Error messages now show "invalid <rule_name>" instead of listing individual expected terminals

- **Dead rule elimination disabled** — all user-defined rules are now kept in the IR since each rule generates a `parse_<name>` public API entry point, even if the rule body was inlined into callers

### Performance

- **JSON `ws` rule**: `repeat(0.., alt((literal(" "), literal("\t"), ...)))` → `take_while(0.., (' ', '\t', '\n', '\r'))` — single SIMD-accelerated call replaces 4 `literal()` + `alt()` + `repeat().fold()`
- **JSON `hex` rule**: `alt((one_of('0'..='9').context(...), one_of('a'..='f').context(...), ...))` → `one_of(('0'..='9', 'a'..='f', 'A'..='F'))` — single call replaces 3 `one_of()` + 3 `.context()` + `alt()`
- **`.context()` reduction**: Internal rules no longer emit `.context()` calls — the primary bottleneck identified in earlier profiling (7x overhead)
- **Trivial rule inlining**: `alpha`, `digit`, `hex` etc. are inlined at IR level, eliminating function call + context push overhead

## [0.1.2] - 2026-04-06

### Added

- **Error messages** with accurate source positions
  - `LineIndex` in runtime crate: memchr-accelerated newline index with O(log n) binary search for byte-offset → line:col conversion
  - `StrContext::Label` on every rule for "invalid <rule>" messages
  - `StrContext::Expected` on string literals and char ranges for "expected ..." messages
  - `furthest_pos` tracking in `ParseState` to report the actual failure position after backtracking
  - Errors now format as `parse error at 3:12: invalid value\nexpected "null", "true", "false"` instead of raw `ContextError { context: [], cause: None }`

- **`memchr` dependency** added to faputa runtime for SIMD-accelerated newline scanning

### Fixed

- Error position no longer reports `1:1` for all failures — uses furthest position reached during parsing

## [0.1.1] - 2026-04-06

### Added

- **Runtime crate** (`faputa`): winnow-based parser runtime
  - `State` trait with flag/counter accessors and line position helpers
  - `Input` type alias wrapping `winnow::stream::Stateful<LocatingSlice<&str>, S>`

- **Code generator** (`faputa_generator`): produces Rust + winnow parser code from AST
  - Per-rule entry points: `parse_<rule>(source) -> Result<&str, String>`
  - Automatic `alt()` chunking for >21 branches (winnow tuple limit)
  - Type unification via `.void()` on choice branches and `.fold()` on repeats
  - Full stateful codegen: `with`/`when`/`guard`/`depth_limit` expressions
  - `generate()` for build.rs (pub mod), `generate_with_mod()` for derive (hidden mod)

- **Derive macro** (`faputa_derive`): `#[derive(Parser)]` proc macro
  - `#[grammar("path")]` to load from file
  - `#[grammar_inline("...")]` for inline grammars
  - Generates hidden module + `impl StructName` with `parse_<rule>()` methods

- **Examples**
  - `examples/parse_demo`: assignment parser, reads from file
  - `examples/parse_json`: full JSON (RFC 8259) grammar and file parser

- **Benchmarks** (`benches/json_bench`): criterion benchmarks comparing faputa vs pest vs serde_json

- **winnow `simd` feature** enabled for memchr-accelerated literal matching

### Changed

- Fixture files moved to workspace root `fixtures/` for shared access across crates
- End-to-end tests (`tests/e2e`) use build.rs codegen with prettyplease formatting

## [0.1.0] - 2026-04-06

Initial release of the faputa meta-compiler pipeline (`faputa_meta`).

### Added

- **Lexer** (`faputa_meta::lexer`): Logos-based tokenizer for `.faputa` grammar files
  - All keywords, operators, delimiters, and built-in predicates (`SOI`, `EOI`, `ANY`, `LINE_START`, `LINE_END`)
  - String literals with escape sequences (`\n`, `\t`, `\r`, `\\`, `\"`)
  - Char literals with escape sequences for char ranges (`'a'..'z'`)
  - Line comments (`//`) and block comments (`/* */`)
  - Unrecognized characters reported as errors with position

- **AST** (`faputa_meta::ast`): Full abstract syntax tree for the `.faputa` DSL
  - State declarations: `let flag`, `let counter`
  - Rule definitions with guard statements, emit statements, and expressions
  - Expressions: string literals, char ranges, identifiers, sequences, choices, repetition (`+`, `*`, `?`, `{n,m}`), positive/negative lookahead, grouping, `with` (flag/counter increment), `when` conditionals, `depth_limit`

- **Parser** (`faputa_meta::parser`): Hand-written recursive descent parser
  - Parses tokenized `.faputa` source into the AST
  - Operator precedence: choice (`|`) < sequence (whitespace) < postfix (`+*?{n,m}`) < prefix (`&!`) < atoms
  - Save/restore backtracking for ambiguous `{` (repeat bounds vs. block)
  - Error reporting with source offset

- **Validator** (`faputa_meta::validator`): Semantic validation pass
  - Duplicate rule and state detection
  - Undefined rule and state reference checking
  - State kind verification (flag vs. counter usage mismatch)
  - Built-in name shadowing prevention

- **Convenience API** (`faputa_meta::compile`): Combined parse + validate entry point

- **Workspace structure**: Multi-crate workspace (`faputa`, `faputa_meta`, `faputa_generator`, `faputa_derive`, `faputa_vm`, `faputa_debugger`, `xtask`)

- **DSL specification**: `docs/faputa-spec.md` with full syntax reference and winnow codegen mapping
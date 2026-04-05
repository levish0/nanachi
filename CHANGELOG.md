# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026-04-06

Initial release of the nanachi meta-compiler pipeline (`nanachi_meta`).

### Added

- **Lexer** (`nanachi_meta::lexer`): Logos-based tokenizer for `.nanachi` grammar files
  - All keywords, operators, delimiters, and built-in predicates (`SOI`, `EOI`, `ANY`, `LINE_START`, `LINE_END`)
  - String literals with escape sequences (`\n`, `\t`, `\r`, `\\`, `\"`)
  - Char literals with escape sequences for char ranges (`'a'..'z'`)
  - Line comments (`//`) and block comments (`/* */`)
  - Unrecognized characters reported as errors with position

- **AST** (`nanachi_meta::ast`): Full abstract syntax tree for the `.nanachi` DSL
  - State declarations: `let flag`, `let counter`
  - Rule definitions with guard statements, emit statements, and expressions
  - Expressions: string literals, char ranges, identifiers, sequences, choices, repetition (`+`, `*`, `?`, `{n,m}`), positive/negative lookahead, grouping, `with` (flag/counter increment), `when` conditionals, `depth_limit`

- **Parser** (`nanachi_meta::parser`): Hand-written recursive descent parser
  - Parses tokenized `.nanachi` source into the AST
  - Operator precedence: choice (`|`) < sequence (whitespace) < postfix (`+*?{n,m}`) < prefix (`&!`) < atoms
  - Save/restore backtracking for ambiguous `{` (repeat bounds vs. block)
  - Error reporting with source offset

- **Validator** (`nanachi_meta::validator`): Semantic validation pass
  - Duplicate rule and state detection
  - Undefined rule and state reference checking
  - State kind verification (flag vs. counter usage mismatch)
  - Built-in name shadowing prevention

- **Convenience API** (`nanachi_meta::compile`): Combined parse + validate entry point

- **Workspace structure**: Multi-crate workspace (`nanachi`, `nanachi_meta`, `nanachi_generator`, `nanachi_derive`, `nanachi_vm`, `nanachi_debugger`, `xtask`)

- **DSL specification**: `docs/nanachi-spec.md` with full syntax reference and winnow codegen mapping
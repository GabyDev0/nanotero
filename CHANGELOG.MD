# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepsachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.1] - 2026-06-27

### Added
- **`TeroDeserialize` Derive Macro:** Introduced a core procedural macro (`#[derive(TeroDeserialize)]`) to automatically map `.tero` configuration fields directly into type-safe Rust structures.
- **`#[tero(name = "...")]` Attribute:** Support for custom field renaming during macro derivation.
- **`TeroError` Enum:** A brand new unified error management architecture that categorizes failures into precise `Lexical`, `Syntactic`, and `Eval` variants.

### Changed
- **Type-Safe API Engine:** Refactored `from_str` to return any type implementing `Deserialize` via type inference (`from_str::<T>`) instead of forcing a generic `Value` data tree.
- **Crate Unification:** Merged the old internal `lex` and `parser` sub-crates into a single, high-performance compiler unit (`compiler`) to optimize build times and streamline internal state management.

### Fixed
- **Error Diagnostics Reporting:** Overhauled the error formatting logic. Syntax and logical errors are now fully readable, descriptive, and explicit instead of swallowing context.
- **Parser Edge-Case Logic:** Fixed a critical bug in the recursive descent structure that previously caused incorrect token evaluations under specific trailing layouts.

### Removed
- **`LoomError`:** Completely removed the old, ambiguous error type in favor of the new structured `TeroError`.
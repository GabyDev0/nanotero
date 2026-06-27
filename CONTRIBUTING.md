# Contributing to NanoTero 🐦

Thank you for your interest in improving NanoTero! All contributions are welcome, whether it's reporting a bug, suggesting a feature, or improving the documentation.

---

## 🐛 How to Report an Issue

If you find a bug or the parser fails with a valid syntax:

1. Check the **Issues** tab to see if someone has already reported it.
2. If not, open a new Issue describing:
* The `.tero` code that caused the failure.
* The exact error returned by the library (lexical or syntactic).
* The behavior you expected.



---

## 🛠️ How to Propose Changes (Pull Requests)

If you want to get your hands dirty and program an improvement, follow these steps:

1. **Fork** the repository to your own account.
2. Create a branch for your feature:

```bash
git checkout -b feat/new-feature

```

*(Or `fix/bug-name` if you are fixing an error).*
3. Write your code following the project guidelines (see below).
4. Make sure to add **tests** that prove your change works.
5. Open a **Pull Request** targeting the `main` branch of the original repository.

---

## 📜 Code Rules (Rust Style)

To keep the codebase clean and efficient, your code must meet the following requirements before submitting the PR:

* **Formatting:** Always run the official formatter to keep the style identical to the rest of the project:

```bash
cargo fmt

```

* **Quality:** Run the Rust linter to ensure there are no bad practices or warnings:

```bash
cargo clippy

```

* **Stability:** All existing tests (and the new ones you add) must pass without any errors:

```bash
cargo test

```

Thank you for making NanoTero a faster and more stable parser for everyone! 🚀
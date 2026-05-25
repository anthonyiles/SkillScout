# Contributing to SkillScout

Thank you for your interest in contributing! This document covers how to report bugs, suggest features, and submit pull requests.

---

## Reporting Bugs

Before opening a new issue, please search existing issues to avoid duplicates.

When filing a bug report, include:

- Your OS and version (Windows 11, macOS 14, Ubuntu 24.04, etc.)
- The SkillScout version or commit hash
- Steps to reproduce
- What you expected to happen vs. what actually happened
- Any relevant logs or error messages from the console

---

## Suggesting Features

Open an issue with the `enhancement` label and describe:

- The problem you're trying to solve
- Your proposed solution or behaviour
- Any alternatives you considered

For larger changes, please open an issue to discuss the approach before writing code — it avoids wasted effort if the direction doesn't fit the project.

---

## Submitting Pull Requests

### 1. Set up the development environment

See the [README](README.md) for prerequisites and local setup instructions.

### 2. Fork and branch

```bash
git checkout -b feature/your-feature-name
# or
git checkout -b fix/short-description
```

### 3. Make your changes

- Keep changes focused — one concern per PR.
- Follow the existing code style (TypeScript + Vue 3 `<script setup>` on the frontend; idiomatic Rust on the backend).
- Do not commit `.env` files or any secrets.

### 4. Test your changes

```bash
# Type-check the frontend
npm run build

# Check and test the Rust backend (run inside src-tauri/)
cargo check
cargo test
```

Run `npm run tauri dev` and manually verify the affected feature works end-to-end before submitting.

### 5. Open the pull request

- Target the `master` branch.
- Write a clear title and description explaining _what_ changed and _why_.
- Reference any related issues (e.g. `Closes #42`).

---

## Code Style

| Layer | Style |
|-------|-------|
| TypeScript / Vue | Follow existing patterns; no linter is enforced yet, but keep formatting consistent with surrounding code |
| Rust | `cargo fmt` before committing; `cargo clippy` warnings should be clean |
| Commits | Short imperative subject line (`fix: …`, `feat: …`, `refactor: …`) |

---

## License

By contributing, you agree that your contributions will be licensed under the project's [MIT License](LICENSE).

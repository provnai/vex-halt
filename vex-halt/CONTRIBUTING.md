# Contributing to VEX-HALT

Thank you for your interest in contributing to VEX-HALT! This document provides guidelines for contributing to the project.

## 🚀 Quick Start

1. **Fork** the repository
2. **Clone** your fork: `git clone https://github.com/YOUR-USERNAME/vex-halt`
3. **Build**: `cargo build --release`
4. **Test**: `cargo test`

## 📋 Guidelines

### Code Style

- Run `cargo fmt` before committing
- Run `cargo clippy` and address warnings
- Follow Rust naming conventions

### Commit Messages

Use conventional commits:
```
feat: add new test category
fix: correct scoring calculation
docs: update README
test: add integration tests
```

### Pull Requests

1. Create a feature branch: `git checkout -b feature/my-feature`
2. Make your changes
3. Ensure tests pass: `cargo test`
4. Submit a PR with a clear description

## 🧪 Adding Test Cases

To add new test items:

1. Create or edit JSON files in `datasets/vex_halt/<category>/`
2. Follow the existing schema for that category
3. Test locally: `cargo run -- --categories <CATEGORY> --provider mock`

## 📝 Reporting Issues

When reporting bugs, please include:
- Rust/Cargo version (`rustc --version`)
- OS and version
- Steps to reproduce
- Expected vs actual behavior

## 💬 Questions?

Open a [Discussion](https://github.com/provnai/vex-halt/discussions) or reach out at [provnai.dev](https://provnai.dev).

---

By contributing, you agree that your contributions will be licensed under the MIT License.

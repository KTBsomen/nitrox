# Contributing to RustyX

First off, thank you for considering contributing to RustyX! It's people like you that make RustyX such a great tool.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [How Can I Contribute?](#how-can-i-contribute)
- [Development Setup](#development-setup)
- [Style Guidelines](#style-guidelines)
- [Commit Messages](#commit-messages)
- [Pull Request Process](#pull-request-process)
- [Community](#community)

## Code of Conduct

This project and everyone participating in it is governed by our Code of Conduct. By participating, you are expected to uphold this code. Please report unacceptable behavior to [bilalmalik1561@gmail.com](mailto:bilalmalik1561@gmail.com).

### Our Standards

- Be respectful and inclusive
- Welcome newcomers and help them learn
- Accept constructive criticism gracefully
- Focus on what is best for the community
- Show empathy towards other community members

## Getting Started

### Prerequisites

- Rust 1.70 or higher
- Cargo (comes with Rust)
- Git

### Fork and Clone

1. Fork the repository on GitHub
2. Clone your fork locally:
   ```bash
   git clone https://github.com/Mohammad007/rustyx.git
   cd rustyx
   ```
3. Add the upstream remote:
   ```bash
   git remote add upstream https://github.com/Mohammad007/rustyx.git
   ```

## How Can I Contribute?

### Reporting Bugs

Before creating bug reports, please check existing issues. When creating a bug report, include:

- **Clear title** describing the issue
- **Steps to reproduce** the behavior
- **Expected behavior** vs actual behavior
- **Environment details** (OS, Rust version, etc.)
- **Code samples** if applicable

### Suggesting Features

We love feature suggestions! Please:

1. Check if the feature already exists or is planned
2. Open an issue with the "feature request" label
3. Describe the feature and its use case
4. Explain why it would be useful

### Code Contributions

#### Good First Issues

Look for issues labeled `good first issue` - these are great for newcomers!

#### Types of Contributions

- 🐛 Bug fixes
- ✨ New features
- 📚 Documentation improvements
- 🧪 Test coverage
- 🎨 Code refactoring
- ⚡ Performance improvements

## Development Setup

### 1. Install Dependencies

```bash
# Clone the repo
git clone https://github.com/Mohammad007/rustyx.git
cd rustyx

# Build the project
cargo build

# Run tests
cargo test
```

### 2. Running Examples

```bash
# Run the basic API example
cargo run --example basic_api

# Run with specific features
cargo run --example basic_api --features "postgres"
```

### 3. Testing

```bash
# Run all tests
cargo test

# Run tests with all features
cargo test --all-features

# Run specific test
cargo test test_name

# Run with output
cargo test -- --nocapture
```

### 4. Documentation

```bash
# Generate docs
cargo doc --open

# Check doc coverage
cargo doc --no-deps
```

## Style Guidelines

### Rust Code Style

We follow the official Rust style guidelines. Use `rustfmt` and `clippy`:

```bash
# Format code
cargo fmt

# Run linter
cargo clippy -- -D warnings
```

### Code Guidelines

1. **Write idiomatic Rust** - Follow Rust conventions
2. **Document public APIs** - Use `///` doc comments
3. **Write tests** - Aim for good test coverage
4. **Handle errors gracefully** - Use `Result` and `?` operator
5. **Avoid `unwrap()`** - Use proper error handling in library code

## Commit Messages

We follow [Conventional Commits](https://www.conventionalcommits.org/):

```
<type>(<scope>): <description>

[optional body]

[optional footer(s)]
```

### Types

- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `style`: Code style changes (formatting, etc.)
- `refactor`: Code refactoring
- `test`: Adding or updating tests
- `chore`: Maintenance tasks
- `perf`: Performance improvements

### Examples

```bash
feat(router): add support for route groups
fix(response): correct JSON content-type header
docs(readme): add database configuration examples
test(middleware): add tests for CORS middleware
refactor(db): simplify connection pooling logic
```

## Pull Request Process

### Before Submitting

1. ✅ Update your fork with the latest upstream changes
2. ✅ Run `cargo fmt` to format your code
3. ✅ Run `cargo clippy` and fix any warnings
4. ✅ Run `cargo test` and ensure all tests pass
5. ✅ Add tests for new functionality
6. ✅ Update documentation if needed
7. ✅ Update the CHANGELOG if applicable

### Submitting

1. Push your changes to your fork
2. Create a Pull Request against `main`
3. Fill out the PR template completely
4. Link any related issues

## Community

### Getting Help

- 📖 Read the [documentation](https://docs.rs/rustyx)
- 💬 Join our [Discord server](https://discord.gg/rustyx)
- 🐦 Follow us on [Twitter](https://twitter.com/rustyx)
- 📧 Email us at [bilalmalik1561@gmail.com](mailto:bilalmalik1561@gmail.com)

### Recognition

Contributors are recognized in:
- The CONTRIBUTORS.md file
- Release notes
- Our website's contributors page

---

## Thank You!

Your contributions make RustyX better for everyone. Whether it's fixing a typo, adding a feature, or improving documentation - every contribution matters!

🚀 Happy coding!

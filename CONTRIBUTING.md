# Contributing to Plasmate

Thanks for your interest in contributing to Plasmate! We welcome contributions of all kinds.

## Quick Links

- [Good First Issues](https://github.com/plasmate-labs/plasmate/labels/good%20first%20issue)
- [Documentation](https://docs.plasmate.app)
- [W3C Community Group](https://www.w3.org/community/web-content-browser-ai/)

## Ways to Contribute

### Report Bugs
Found a URL that Plasmate doesn't handle correctly? Open an issue with:
- The URL
- Expected output
- Actual output
- Your Plasmate version (`plasmate --version`)

### Improve Coverage
Plasmate's SOM coverage grows with every website we test against. You can help by:
- Testing Plasmate against websites you use
- Reporting missing elements or incorrect parsing
- Submitting PRs that improve SOM extraction for specific HTML patterns

### Add Tests
More tests = more confidence. Our test suite lives alongside the source code.

### Improve Documentation
- Fix typos or unclear explanations
- Add examples
- Translate docs

### Build Integrations
We maintain integration packages for popular frameworks. See [awesome-plasmate](https://github.com/plasmate-labs/awesome-plasmate) for the full list.

## Development Setup

```bash
git clone https://github.com/plasmate-labs/plasmate
cd plasmate
cargo build
cargo test
cargo run -- fetch https://example.com
```

## Pull Request Process

1. Fork the repo and create a branch from `master`
2. Make your changes
3. Run `cargo test` and ensure all tests pass
4. Run `cargo clippy` for linting
5. Update documentation if needed
6. Submit a PR with a clear description

## Code Style

- Follow standard Rust conventions
- Use `cargo fmt` before committing
- Keep functions focused and well-documented
- Add tests for new functionality

## License

By contributing, you agree that your contributions will be licensed under the Apache 2.0 License.

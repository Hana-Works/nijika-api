# Contributing to Nijika API

Thank you for your interest in contributing to Nijika API! We welcome contributions from the community.

## Getting Started

1.  **Fork the repository** on Codeberg.
2.  **Clone your fork** locally:
    ```bash
    git clone ssh://git@codeberg.org/your-username/nijika-api.git
    cd nijika-api
    ```
3.  **Create a branch** for your feature or bug fix:
    ```bash
    git checkout -b feature/amazing-feature
    ```

## Development Standards

### Automation
We provide a `Makefile` to simplify local development. You can run all standard checks with:
```bash
make check
```

### Code Style
- We use standard Rust formatting. Run `make fmt` (or `cargo fmt`) before committing.
- Ensure your code passes clippy checks: `make clippy` (or `cargo clippy`).
- **Note:** CI will fail if the code is not formatted or has clippy warnings. We no longer automatically fix formatting in CI to keep the git history clean.

### Documentation
- Update `README.md` if you change behavior or configuration.
- Add Rustdoc comments (`///`) for public functions and modules.
- Update `CHANGELOG.md` with your changes under `[Unreleased]`.

### Testing
- Add unit tests for new logic.
- Run all tests before pushing: `cargo test`.

## Pull Request Process

1.  Push your branch to your fork.
2.  Open a Pull Request (PR) against the `main` branch.
3.  Describe your changes clearly in the PR description.
4.  Link any relevant issues (e.g., "Closes #123").
5.  Wait for review and address any feedback.

## Code of Conduct

Please be respectful and professional in all interactions.

# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Changed
- Updated `.gitignore` to include Python-specific patterns and Gemini CLI temporary files.
- Refactored `removebg` handler to stream responses from Modal worker instead of buffering, improving memory usage and latency.

### Added
- Initial project structure with Rust (2024 edition).
- Basic web server using Axum and Tokio.
- Health check endpoint (`GET /health`).
- Documentation framework (`README.md`, Rustdoc).
- Quality Audit Report.
- Expanded documentation: `docs/architecture.md` and `docs/api.md`.
- Documentation quality improvements (configuration details, design decisions).
- Documented `src/models/mod.rs`.
- Centralized configuration management in `src/config.rs` using `dotenvy`.
- Added `workers/README.md` for Modal worker deployment instructions.
- Updated `README.md` and `docs/api.md` to include `removebg` feature and configuration.
- Integration tests for rate limiting.

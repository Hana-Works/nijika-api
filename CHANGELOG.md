# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.1] - 2026-02-06

### Added
- Forgejo CI/CD workflow for automated code quality checks (fmt, clippy), testing, and release builds.
- Image upscaler endpoint (`POST /upscale`) using Real-ESRGAN.
- Upscaler Modal worker in `workers/upscaler/`.
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

### Changed
- Migrated primary repository to Codeberg: `ssh://git@codeberg.org/hanaworks-opensource-project/nijika-api.git`.
- Updated `README.md` and `CONTRIBUTING.md` with new repository links and updated feature descriptions.
- Concurrency support for Modal workers (`removebg` and `upscaler`) using `allow_concurrent_inputs`, enabling multiple requests to be processed by a single GPU instance.
- Async image fetching in Modal workers using `httpx` for better resource utilization.
- Thread-safe model caching in the upscaler worker to support concurrent requests.
- Updated `.env.example` to include all currently used environment variables (`MODAL_UPSCALER_URL`, `RATE_LIMIT_PER_SECOND`, `RATE_LIMIT_BURST`).
- Upgraded Modal workers to use NVIDIA L4 GPUs (from T4) for better performance and cost-efficiency.
- Increased concurrency limits for workers: `removebg` now supports 8 concurrent inputs and `upscaler` supports 4.
- Updated `.gitignore` to include Python-specific patterns and Gemini CLI temporary files.
- Refactored `removebg` handler to stream responses from Modal worker instead of buffering, improving memory usage and latency.
- Optimized Modal workers (`removebg` and `upscaler`) by baking default models into container images, eliminating model downloads on every cold start and improving response times.

### Fixed
- Fixed integration tests (`heavy_load_test.rs` and `rate_limit_test.rs`) that were failing to compile due to missing fields in `Config` initialization.

## [Unreleased]

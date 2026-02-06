# Nijika API

## Project Overview
`nijika-api` is a high-performance web API built with Rust (2024 edition). It serves as a gateway to AI-powered image processing services, specifically background removal and image upscaling, utilizing serverless GPU workers on Modal.

## Technologies
- **Language:** Rust (Edition 2024)
- **Web Framework:** Axum
- **Async Runtime:** Tokio
- **Logging:** Tracing
- **Workers:** Python + Modal (Serverless GPU)

## Project Structure
- `src/main.rs`: Entry point, server initialization.
- `src/lib.rs`: Library root, router definition.
- `src/handlers/`: Business logic for API endpoints (`removebg`, `upscale`).
- `src/models/`: Request and response data structures.
- `src/config.rs`: Centralized configuration management.
- `workers/`: Modal worker implementations (Python).
- `docs/`: Detailed API and architecture documentation.

## Core Features
- **Background Removal:** `/removebg` endpoint using BiRefNet.
- **Image Upscaling:** `/upscale` endpoint using Real-ESRGAN.
- **Rate Limiting:** Integrated request throttling.
- **Multipart & JSON Support:** Flexible input options for both URL and file uploads.

## Building and Running
```bash
# Build
cargo build

# Run
cargo run

# Test
cargo test
```

## Development Conventions
- **Formatting:** `cargo fmt`
- **Linting:** `cargo clippy`
- **Changelog:** Maintain `CHANGELOG.md` following [Keep a Changelog](https://keepachangelog.com/).
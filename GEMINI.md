# Nijika API

## Project Overview
`nijika-api` is a high-performance web API built with Rust (2024 edition). It serves as a gateway to AI-powered image processing services, specifically background removal and image upscaling, utilizing serverless GPU workers on Modal. It includes a user management system with OAuth authentication and a credit-based usage model.

## Technologies
- **Language:** Rust (Edition 2024)
- **Web Framework:** Axum
- **Async Runtime:** Tokio
- **Database:** PostgreSQL + SQLx
- **Authentication:** OAuth2 (GitHub, GitLab) + Session-based auth
- **Templating:** Askama (HTML)
- **Logging:** Tracing
- **Workers:** Python + Modal (Serverless GPU)

## Project Structure
- `src/main.rs`: Entry point, server initialization.
- `src/lib.rs`: Library root.
- `src/handlers/`: Business logic for API endpoints and auth.
- `src/routes/`: Route definitions and middleware.
- `src/models/`: Data structures and database models.
- `src/config.rs`: Centralized configuration management.
- `templates/`: HTML templates for the web interface.
- `migrations/`: Database migration files.
- `workers/`: Modal worker implementations (Python).
- `docs/`: Detailed API and architecture documentation.

## Core Features
- **Background Removal:** `/api/removebg` endpoint using BiRefNet (Cost: 0.01 credits).
- **Image Upscaling:** `/api/upscale` endpoint using Real-ESRGAN (Cost: 0.02 credits).
- **User Authentication:** OAuth2 integration with GitHub and GitLab.
- **Anti-Abuse Logic:** 1-month account age requirement for OAuth logins.
- **Credits System:** Usage-based billing with a 50-credit registration bonus.
- **Rate Limiting:** Multi-layered request throttling using `lazy-limit` and `axum-governor`.
- **Web Dashboard:** User interface for managing API keys, viewing logs, and linked accounts.
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
# Nijika API

`nijika-api` is a high-performance web API built with Rust (2024 edition). It serves as a gateway to AI-powered image processing services, utilizing serverless GPU workers on Modal, featuring a credit-based usage model and OAuth2 authentication.

## Features

- **Background Removal:** AI-powered background removal using BiRefNet.
- **Image Upscaling:** AI-powered upscaling using Real-ESRGAN.
- **User Management:** OAuth2 authentication with GitHub and GitLab.
- **Credit System:** Usage-based billing/quota system.
- **Web Dashboard:** Simple UI for managing API keys and credits.
- **Rate Limiting:** Declarative rate limiting on a per-endpoint basis.
- **High Performance:** Built with Axum, Tokio, and Rust.
- **Streaming:** Efficient streaming of processed images.

## Technology Stack

- **Backend:** Rust (Axum, Tokio, SQLx)
- **Database:** PostgreSQL
- **Frontend:** Askama (HTML templates)
- **Workers:** Python (Modal, PyTorch)
- **Containerization:** Docker

## Getting Started

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (2024 edition)
- [PostgreSQL](https://www.postgresql.org/)
- [Modal Token](https://modal.com/docs/guide/token) (for running workers)

### Installation

1.  **Clone the repository:**
    ```bash
    git clone ssh://git@codeberg.org/hanaworks-opensource-project/nijika-api.git
    cd nijika-api
    ```

2.  **Set up environment variables:**
    Copy `.env.example` to `.env` and fill in your credentials.
    ```bash
    cp .env.example .env
    ```

3.  **Run migrations:**
    ```bash
    cargo run # Migrations are run automatically on startup
    ```

### Using the Makefile

We provide a `Makefile` for common development tasks:

- `make check`: Run formatting, linting, and tests.
- `make build`: Build the project in release mode.
- `make run`: Run the development server.
- `make fmt`: Format code.
- `make clippy`: Run linter.

## Configuration

| Variable | Description | Default |
|----------|-------------|---------|
| `DATABASE_URL` | PostgreSQL connection string | (Required) |
| `GITHUB_CLIENT_ID` | GitHub OAuth Client ID | (Required) |
| `GITHUB_CLIENT_SECRET` | GitHub OAuth Client Secret | (Required) |
| `GITLAB_CLIENT_ID` | GitLab OAuth Client ID | (Required) |
| `GITLAB_CLIENT_SECRET` | GitLab OAuth Client Secret | (Required) |
| `BASE_URL` | Base URL for OAuth callbacks | `http://localhost:3000` |
| `MODAL_REMOVEBG_URL` | URL of the Background Removal worker | (Required) |
| `MODAL_UPSCALER_URL` | URL of the Upscaler worker | (Required) |

## API Documentation

The API endpoints are prefixed with `/api`. Authentication is required via the `X-API-Key` header.

- **POST** `/api/removebg`: Remove image background.
- **POST** `/api/upscale`: Upscale and enhance images.

For full details, see the [API Reference](docs/api.md).

## Architecture

For a deep dive into the system design, see [Architecture Overview](docs/architecture.md).

## License

This project is licensed under the [MIT License](LICENSE).
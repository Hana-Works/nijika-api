# Nijika API

A Rust-based web API built with Axum, Tokio, and Tracing.

## Features

- **High Performance:** Built on top of Axum and Tokio.
- **Observability:** Integrated tracing for logging and diagnostics.
- **Simple Architecture:** Clean separation of concerns (Routes, Handlers, Models).
- **Background Removal:** AI-powered background removal using BiRefNet on Modal.
- **Image Upscaling:** AI-powered upscaling using Real-ESRGAN on Modal.

## Quick Start

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (2024 edition)

### Running the Application

1.  **Clone the repository:**
    ```bash
    git clone ssh://git@codeberg.org/hanaworks-opensource-project/nijika-api.git
    cd nijika-api
    ```

2.  **Set up environment variables:**
    Copy `.env.example` to `.env` (if available) or create one.
    ```bash
    cp .env.example .env
    ```

3.  **Run the server:**
    ```bash
    cargo run
    ```

4.  **Test the health check:**
    ```bash
    curl http://127.0.0.1:3000/health
    ```

## Configuration

The application can be configured using environment variables.

| Variable | Description | Default |
|----------|-------------|---------|
| `HOST` | The interface to bind to | `127.0.0.1` |
| `PORT` | The port to listen on | `3000` |
| `RUST_LOG` | Log level (e.g., `info`, `debug`) | `error` (default if unset) |
| `MODAL_REMOVEBG_URL` | URL of the deployed Modal worker | `http://localhost:8000` |
| `MODAL_UPSCALER_URL` | URL of the deployed Upscaler worker | `http://localhost:8001` |
| `RATE_LIMIT_PER_SECOND` | Max requests per second | `50` |
| `RATE_LIMIT_BURST` | Max burst size | `100` |

## Architecture

The project follows a modular structure:
- **Routes:** Route definitions.
- **Handlers:** Business logic.
- **Models:** Data structures.

For a detailed overview, see [Architecture Overview](docs/architecture.md).

## API Reference

For detailed endpoint documentation, see the [API Reference](docs/api.md).

### Health Check

**GET** `/health`

Returns the status of the API.

**Response:**

- **200 OK**
  ```json
  {
    "status": "ok"
  }
  ```

## Development

### Running Tests

```bash
cargo test
```

### Formatting & Linting

```bash
cargo fmt
cargo clippy
```

## License

[MIT](LICENSE)

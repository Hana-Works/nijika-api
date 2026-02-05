# Nijika API

A Rust-based web API built with Axum, Tokio, and Tracing.

## Features

- **High Performance:** Built on top of Axum and Tokio.
- **Observability:** Integrated tracing for logging and diagnostics.
- **Simple Architecture:** Clean separation of concerns (Routes, Handlers, Models).

## Quick Start

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (2024 edition)

### Running the Application

1.  **Clone the repository:**
    ```bash
    git clone <repository_url>
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

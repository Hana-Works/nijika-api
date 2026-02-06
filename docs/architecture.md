# Architecture Overview

This document describes the high-level architecture of the `nijika-api`.

## System Overview

`nijika-api` is a modular web API built using the Rust programming language and the Axum framework. It follows a clear separation of concerns to ensure maintainability and scalability.

## Core Components

The application is structured into several key modules:

- **`main.rs`**: The entry point. It handles environment configuration, tracing initialization, and server startup.
- **`lib.rs`**: The library crate root. It exposes the main router and internal modules.
- **`routes/`**: Defines the API's URL structure and maps routes to their respective handlers.
- **`handlers/`**: Contains the business logic for processing requests and generating responses.
- **`models/`**: Defines the data structures (schemas) used throughout the application, including database models and request/response DTOs.

## External Services

- **Modal Worker**: A Python-based serverless worker (hosted on Modal.com) responsible for GPU-intensive tasks like background removal. The Rust API acts as a proxy/gateway to this service.

## Data Flow

1.  **Request**: A client sends an HTTP request to the server.
2.  **Routing**: The Axum router matches the request path and method to a handler defined in the `routes` module.
3.  **Handling**: The handler in the `handlers` module receives the request (and any extracted data). It may interact with services or models to perform business logic.
    - *Example*: For `/removebg`, the handler forwards the request to the Modal worker via HTTP.
4.  **Modeling**: Data is structured using types defined in the `models` module.
5.  **Response**: The handler returns a response. For resource-intensive tasks, the response from the Modal worker is streamed back to the client to minimize memory overhead.

## Design Decisions

### Why Modal for Workers?
We utilize [Modal](https://modal.com) for GPU-intensive tasks like background removal for several reasons:
- **Serverless GPU**: Eliminates the need to manage permanent GPU infrastructure, reducing costs significantly for sporadic workloads.
- **Scalability**: Modal scales from 0 to many workers automatically based on demand.
- **Python Ecosystem**: Allows leveraging the rich ecosystem of Python AI/ML libraries (like PyTorch and BiRefNet) while keeping the main API in high-performance Rust.
- **Concurrency Support**: Workers are configured with `allow_concurrent_inputs`, enabling a single GPU instance to process multiple requests simultaneously, maximizing resource utilization.

### Why Rust & Axum?
- **Performance**: Rust provides near-native performance with memory safety guarantees.
- **Reliability**: Axum's type-safe routing and error handling minimize runtime exceptions.
- **Concurrency**: Tokio's async runtime handles high concurrency efficiently, ideal for an API gateway.
- **Streaming Support**: Axum and `reqwest` allow for efficient streaming of large binary payloads (like processed images) directly from the backend worker to the client.

## Technology Stack

- **Language**: [Rust](https://www.rust-lang.org/) (2024 edition)
- **Web Framework**: [Axum](https://github.com/tokio-rs/axum)
- **Asynchronous Runtime**: [Tokio](https://tokio.rs/)
- **Logging & Diagnostics**: [Tracing](https://github.com/tokio-rs/tracing)
- **Environment Management**: [dotenvy](https://github.com/allan2/dotenvy)
- **Serialization**: [Serde](https://serde.rs/)
- **Rate Limiting**: [tower-governor](https://github.com/benwis/tower-governor)
- **HTTP Client**: [reqwest](https://github.com/seanmonstar/reqwest)

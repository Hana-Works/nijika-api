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

## Data Flow

1.  **Request**: A client sends an HTTP request to the server.
2.  **Routing**: The Axum router matches the request path and method to a handler defined in the `routes` module.
3.  **Handling**: The handler in the `handlers` module receives the request (and any extracted data). It may interact with services or models to perform business logic.
4.  **Modeling**: Data is structured using types defined in the `models` module.
5.  **Response**: The handler returns a response, which Axum serializes (typically to JSON) and sends back to the client.

## Technology Stack

- **Language**: [Rust](https://www.rust-lang.org/) (2024 edition)
- **Web Framework**: [Axum](https://github.com/tokio-rs/axum)
- **Asynchronous Runtime**: [Tokio](https://tokio.rs/)
- **Logging & Diagnostics**: [Tracing](https://github.com/tokio-rs/tracing)
- **Environment Management**: [dotenvy](https://github.com/allan2/dotenvy)
- **Serialization**: [Serde](https://serde.rs/)

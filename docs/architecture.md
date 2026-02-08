# Architecture Overview

This document describes the high-level architecture of the `nijika-api`.

## System Overview

`nijika-api` is a modular web API built using the Rust programming language and the Axum framework. It follows a clear separation of concerns to ensure maintainability and scalability.

## Core Components

The application is structured into several key modules:

- **`main.rs`**: The entry point. It handles environment configuration, tracing initialization, database pool creation, and server startup.
- **`lib.rs`**: The library crate root. It defines the `AppState` and exposes internal modules.
- **`routes/`**: Defines the API's URL structure, manages middleware (auth, rate limiting, sessions), and maps routes to handlers.
- **`handlers/`**: Contains the business logic for processing requests (API and web) and generating responses.
- **`models/`**: Defines the data structures (schemas) used throughout the application, including PostgreSQL models (via SQLx) and request/response DTOs.
- **`templates/`**: HTML templates rendered using Askama for the web dashboard and login pages.

## External Services

- **Modal Worker**: A Python-based serverless worker (hosted on Modal.com) responsible for GPU-intensive tasks like background removal. The Rust API acts as a proxy/gateway to this service.
- **OAuth Providers**: GitHub and GitLab are used for user authentication.
- **PostgreSQL**: Stores user data, API keys, and credit balances.

## Data Flow

### Web Authentication Flow
1.  **Login**: User selects an OAuth provider (GitHub/GitLab).
2.  **Redirect**: Server redirects user to the provider's authorization page.
3.  **Callback**: Provider redirects back with an authorization code.
4.  **Exchange**: Server exchanges the code for an access token and fetches user info.
5.  **User Mapping & Linking**:
    - **Existing Provider Link**: If the provider ID (e.g., `github_id`) is already associated with a user, they are logged in.
    - **Account Linking (Logged In)**: If the user is already logged in, the new provider is linked to their current account.
    - **Automatic Linking (Email Match)**: If not logged in, but the provider returns a verified email matching an existing user, the provider is automatically linked to that user.
    - **New User**: If no match is found, a new user is created with initial credits and an API key.
6.  **Session**: A session is created and stored in PostgreSQL, and a session cookie is set in the user's browser.
7.  **Dashboard**: User is redirected to the dashboard to view their API key, credits, and linked accounts.

### API Request Flow
1.  **Request**: A client sends an HTTP request to `/api/*` with an `X-API-Key` header.
2.  **Authentication Middleware**: 
    - Extracts the API key.
    - Validates it against the database.
    - Checks if the user has sufficient credits (> 0).
3.  **Routing**: The Axum router matches the request to a handler.
4.  **Handling**: The handler forwards the request to the Modal worker.
5.  **Response & Credit Deduction**: 
    - The processed image is streamed back to the client.
    - Upon success, credits are deducted from the user's account in the database.

## Design Decisions

### Why Modal for Workers?
We utilize [Modal](https://modal.com) for GPU-intensive tasks like background removal for several reasons:
- **Serverless GPU**: Eliminates the need to manage permanent GPU infrastructure, reducing costs significantly for sporadic workloads.
- **Scalability**: Modal scales from 0 to many workers automatically based on demand.
- **Python Ecosystem**: Allows leveraging the rich ecosystem of Python AI/ML libraries (like PyTorch and BiRefNet) while keeping the main API in high-performance Rust.
- **Concurrency Support**: Workers are configured with `allow_concurrent_inputs`, enabling a single GPU instance to process multiple requests simultaneously, maximizing resource utilization.

### Middleware & Security
We use several layers of middleware to ensure security and reliability:
- **`axum-governor`**: Implements standard in-memory rate limiting for general API and health endpoints.
- **`lazy-limit`**: Used in `main.rs` for global and prefix-based rate limiting with shared state support.
- **`RealIpLayer`**: Ensures accurate rate limiting by extracting the client's true IP address when the API is deployed behind a proxy or load balancer.
- **`#[price("n")]` macro**: A custom attribute macro that handles atomic credit validation, deduction, and automatic refunds upon failure. It ensures business logic remains clean while enforcing payment rules.
- **`PrivateCookieJar`**: Used for secure, encrypted session management via `axum-extra`, providing a lightweight and high-performance alternative to external session stores for user authentication.
- **Anti-Abuse Verification**: Authentication logic includes a check for the account age of the OAuth provider (GitHub/GitLab), requiring accounts to be at least one month old to prevent spam registrations.

### Why Rust & Axum?
- **Performance**: Rust provides near-native performance with memory safety guarantees.
- **Reliability**: Axum's type-safe routing and error handling minimize runtime exceptions.
- **Concurrency**: Tokio's async runtime handles high concurrency efficiently, ideal for an API gateway.
- **Streaming Support**: Axum and `reqwest` allow for efficient streaming of large binary payloads (like processed images) directly from the backend worker to the client.

## Technology Stack

- **Language**: [Rust](https://www.rust-lang.org/) (2024 edition)
- **Web Framework**: [Axum](https://github.com/tokio-rs/axum)
- **Asynchronous Runtime**: [Tokio](https://tokio.rs/)
- **Database**: [PostgreSQL](https://www.postgresql.org/) with [SQLx](https://github.com/launchbadge/sqlx)
- **Authentication**: [OAuth2-rs](https://github.com/ramosbugs/oauth2-rs) and [axum-extra](https://github.com/tokio-rs/axum/tree/main/axum-extra) (PrivateCookieJar)
- **Templating**: [Askama](https://github.com/askama-rs/askama)
- **Logging & Diagnostics**: [Tracing](https://github.com/tokio-rs/tracing)
- **Environment Management**: [dotenvy](https://github.com/allan2/dotenvy)
- **Serialization**: [Serde](https://serde.rs/)
- **Rate Limiting**: [axum-governor](https://github.com/lucacasonato/axum-governor) and [lazy-limit](https://github.com/canmi21/lazy-limit)
- **HTTP Client**: [reqwest](https://github.com/seanmonstar/reqwest)

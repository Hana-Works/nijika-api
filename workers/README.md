# Nijika Workers

This directory contains the background processing workers for the Nijika API, powered by [Modal](https://modal.com/).

## Remove Background Worker (`removebg`)

A high-performance background removal service using `BiRefNet`, capable of running on GPU endpoints.

### Prerequisites

- Python 3.11+
- A [Modal](https://modal.com/) account
- `modal` CLI installed and authenticated

### Setup

1.  **Install dependencies:**
    ```bash
    pip install modal
    ```

2.  **Authenticate with Modal:**
    ```bash
    modal token new
    ```

### Deployment

To deploy the worker to Modal's serverless infrastructure:

```bash
cd workers/removebg
modal deploy app.py
```

Upon successful deployment, Modal will return a URL (e.g., `https://your-username--nijika-removebg-model-remove.modal.run`).

### Configuration

Copy the provided URL and update your main API configuration:

1.  Open or create your `.env` file in the project root.
2.  Set the `MODAL_REMOVEBG_URL` variable:
    ```env
    MODAL_REMOVEBG_URL=https://your-username--nijika-removebg-model-remove.modal.run
    ```

### Local Development

You can serve the worker locally for development:

```bash
modal serve app.py
```

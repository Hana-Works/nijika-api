# Nijika Workers

This directory contains the background processing workers for the Nijika API, powered by [Modal](https://modal.com/).

## Remove Background Worker (`removebg`)

A high-performance background removal service using `BiRefNet`, capable of running on GPU endpoints.

## Image Upscaler Worker (`upscaler`)

An image restoration and upscaling service using `Real-ESRGAN`, providing high-quality results for various image types.

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

To deploy the workers to Modal's serverless infrastructure:

```bash
# Deploy Remove Background Worker
cd workers/removebg
modal deploy app.py

# Deploy Upscaler Worker
cd ../upscaler
modal deploy app.py
```

Upon successful deployment, Modal will return URLs for each service.

### Configuration

Update your main API configuration in the `.env` file:

1.  Open or create your `.env` file in the project root.
2.  Set the worker URLs:
    ```env
    MODAL_REMOVEBG_URL=https://your-username--nijika-removebg-app-remove.modal.run
    MODAL_UPSCALER_URL=https://your-username--nijika-upscaler-app-upscale.modal.run
    ```

### Local Development

You can serve the workers locally for development:

```bash
# Serve Remove Background Worker
cd workers/removebg
modal serve app.py

# Serve Upscaler Worker
cd ../upscaler
modal serve app.py
```

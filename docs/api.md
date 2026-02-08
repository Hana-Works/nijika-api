# API Reference

This document provides detailed information about the endpoints available in the `nijika-api`.

## Base URL

The API is accessible at:
`http://<host>:<port>`

(Default: `http://127.0.0.1:3000`)

## Endpoints

### Health Check

Returns the current status of the API.

- **URL:** `/health`
- **Method:** `GET`
- **Authentication:** None
- **Success Response:**
    - **Code:** `200 OK`
    - **Content:**
      ```json
      {
        "status": "ok"
      }
      ```

### Authentication

These endpoints are used for user authentication and are accessible via a web browser.

- **GET `/auth/github`**: Initiate GitHub OAuth flow.
- **GET `/auth/gitlab`**: Initiate GitLab OAuth flow.
- **POST `/auth/regenerate-api-key`**: Regenerate the current user's API key. Redirects back to `/dashboard`.
- **GET `/auth/logout`**: Log out and clear session.

### Processing API

All processing endpoints require authentication via an API key.

- **Header:** `X-API-Key: <your_api_key>`
- **Credit Cost:** Each successful request deducts credits from your account.

### Remove Background

Removes the background from an image using an AI model.

- **URL:** `/api/removebg`
- **Method:** `POST`
- **Authentication:** `X-API-Key` header
- **Content-Types:** `application/json` or `multipart/form-data`

#### Option 1: JSON Payload (URL)

Provide an image URL to process.

- **Headers:** 
    - `Content-Type: application/json`
    - `X-API-Key: <your_api_key>`
- **Body:**
  ```json
  {
    "url": "https://example.com/image.jpg"
  }
  ```

#### Option 2: Multipart Upload (File)

Upload an image file directly.

- **Headers:** 
    - `Content-Type: multipart/form-data`
    - `X-API-Key: <your_api_key>`
- **Body:** Form data with a field named `image`.

#### Response

- **Success:**
    - **Code:** `200 OK`
    - **Content-Type:** `image/png`
    - **Body:** Binary PNG image data.

- **Error Response:**
    - **Code:** `400 Bad Request` (Invalid JSON or missing image)
    - **Code:** `401 Unauthorized` (Invalid or missing API key)
    - **Code:** `402 Payment Required` (Insufficient credits)
    - **Code:** `500 Internal Server Error` (Worker connection failure)
    - **Code:** `502 Bad Gateway` (Worker processing error)

### Image Upscaler

Upscales and restores images using Real-ESRGAN.

- **URL:** `/api/upscale`
- **Method:** `POST`
- **Authentication:** `X-API-Key` header
- **Content-Types:** `application/json` or `multipart/form-data`

#### Option 1: JSON Payload (URL)

- **Headers:** 
    - `Content-Type: application/json`
    - `X-API-Key: <your_api_key>`
- **Body:**
  ```json
  {
    "url": "https://example.com/image.jpg",
    "model": "RealESRGAN_x4plus_anime_6B",
    "scale": 4,
    "face_enhance": false
  }
  ```

#### Option 2: Multipart Upload (File)

- **Headers:** 
    - `Content-Type: multipart/form-data`
    - `X-API-Key: <your_api_key>`
- **Body:**
    - `image` (required): Binary image file.
    - `model` (optional): Text field.
    - `scale` (optional): Text field (numeric).
    - `face_enhance` (optional): Text field (`true`/`false`).

#### Response

- **Success:**
    - **Code:** `200 OK`
    - **Content-Type:** `image/jpeg`
    - **Body:** Binary JPEG image data.

- **Error Response:**
    - **Code:** `400 Bad Request`
    - **Code:** `401 Unauthorized`
    - **Code:** `402 Payment Required`
    - **Code:** `500 Internal Server Error`
    - **Code:** `502 Bad Gateway`

## Error Handling

The API uses standard HTTP status codes to indicate the success or failure of a request.

| Status Code | Description |
|-------------|-------------|
| `200 OK` | The request was successful. |
| `400 Bad Request` | The request was invalid or cannot be served. |
| `401 Unauthorized` | Invalid or missing API key. |
| `402 Payment Required` | Insufficient credits. |
| `429 Too Many Requests` | Rate limit exceeded. |
| `404 Not Found` | The requested resource could not be found. |
| `500 Internal Server Error` | An unexpected error occurred on the server. |
| `502 Bad Gateway` | The processing worker (Modal) returned an error or is unreachable. |

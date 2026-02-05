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

### Remove Background

Removes the background from an image using an AI model.

- **URL:** `/removebg`
- **Method:** `POST`
- **Authentication:** None
- **Content-Types:** `application/json` or `multipart/form-data`

#### Option 1: JSON Payload (URL)

Provide an image URL to process.

- **Headers:** `Content-Type: application/json`
- **Body:**
  ```json
  {
    "url": "https://example.com/image.jpg"
  }
  ```

#### Option 2: Multipart Upload (File)

Upload an image file directly.

- **Headers:** `Content-Type: multipart/form-data`
- **Body:** Form data with a field named `image`.

#### Response

- **Success:**
    - **Code:** `200 OK`
    - **Content-Type:** `image/png`
    - **Body:** Binary PNG image data.

- **Error Response:**
    - **Code:** `400 Bad Request` (Invalid JSON or missing image)
    - **Code:** `500 Internal Server Error` (Worker connection failure)
    - **Code:** `502 Bad Gateway` (Worker processing error)

## Error Handling

The API uses standard HTTP status codes to indicate the success or failure of a request.

| Status Code | Description |
|-------------|-------------|
| `200 OK` | The request was successful. |
| `400 Bad Request` | The request was invalid or cannot be served. |
| `404 Not Found` | The requested resource could not be found. |
| `500 Internal Server Error` | An unexpected error occurred on the server. |

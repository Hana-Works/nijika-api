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

## Error Handling

The API uses standard HTTP status codes to indicate the success or failure of a request.

| Status Code | Description |
|-------------|-------------|
| `200 OK` | The request was successful. |
| `400 Bad Request` | The request was invalid or cannot be served. |
| `404 Not Found` | The requested resource could not be found. |
| `500 Internal Server Error` | An unexpected error occurred on the server. |

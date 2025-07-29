# Product Extraction API - cURL Examples

This file contains direct cURL examples for using the product extraction API.

## Prerequisites

1. Start the LLM Web Agent server:
   ```bash
   cargo run --release
   ```

2. Server should be running on `http://localhost:3000`

## Basic Usage

### Health Check

```bash
curl -X GET http://localhost:3000/health | jq .
```

Expected response:
```json
{
  "status": "healthy",
  "message": "LLM Web Agent is running!",
  "active_sessions": 0,
  "memory_usage_mb": 45,
  "timestamp": "2024-01-15T10:30:00Z"
}
```

### Extract Product Information (Temporary Session)

The simplest way to extract product information - the system will create a temporary browser session:

```bash
curl -X POST http://localhost:3000/product/information \
  -H "Content-Type: application/json" \
  -d '{
    "url": "https://www.amazon.com/dp/B08N5WRWNW"
  }' | jq .
```

### Extract Product Information (With Session)

For better performance when making multiple requests, create a session first:

1. **Create a browser session:**
   ```bash
   SESSION_ID=$(curl -s -X POST http://localhost:3000/browser/session | jq -r '.session_id')
   echo "Session ID: $SESSION_ID"
   ```

2. **Use the session for product extraction:**
   ```bash
   curl -X POST http://localhost:3000/product/information \
     -H "Content-Type: application/json" \
     -d '{
       "url": "https://www.amazon.com/dp/B08N5WRWNW",
       "session_id": "'$SESSION_ID'"
     }' | jq .
   ```

3. **Reuse the same session for another product:**
   ```bash
   curl -X POST http://localhost:3000/product/information \
     -H "Content-Type: application/json" \
     -d '{
       "url": "https://www.amazon.com/Star-Wars-Echo-Dot-bundle/dp/B0DZQ92XQZ/",
       "session_id": "'$SESSION_ID'"
     }' | jq .
   ```

## Example Response

```json
{
  "success": true,
  "product": {
    "name": "Amazon Echo Dot (4th Gen, 2020 release) | Smart speaker with Alexa | Charcoal",
    "description": "Smart speaker with Alexa voice control, compact design, premium sound",
    "price": "$49.99",
    "availability": "In Stock",
    "brand": "Amazon",
    "rating": "4.7 out of 5 stars",
    "image_url": "https://m.media-amazon.com/images/I/61H3K9GkbKL._AC_SL1000_.jpg",
    "raw_data": null
  },
  "error": null,
  "extraction_time_ms": 3245
}
```

## Test URLs

Here are some URLs you can test with:

### Amazon Products
```bash
# Echo Dot (4th Gen)
curl -X POST http://localhost:3000/product/information \
  -H "Content-Type: application/json" \
  -d '{"url": "https://www.amazon.com/dp/B08N5WRWNW"}' | jq .

# Star Wars Echo Dot Bundle
curl -X POST http://localhost:3000/product/information \
  -H "Content-Type: application/json" \
  -d '{"url": "https://www.amazon.com/Star-Wars-Echo-Dot-bundle/dp/B0DZQ92XQZ/"}' | jq .

# Kindle Paperwhite
curl -X POST http://localhost:3000/product/information \
  -H "Content-Type: application/json" \
  -d '{"url": "https://www.amazon.com/dp/B08KTZ8249"}' | jq .
```

### Test Page (For Development)
```bash
# Simple HTML page for testing
curl -X POST http://localhost:3000/product/information \
  -H "Content-Type: application/json" \
  -d '{"url": "https://httpbin.org/html"}' | jq .
```

## Error Handling

### Invalid URL
```bash
curl -X POST http://localhost:3000/product/information \
  -H "Content-Type: application/json" \
  -d '{"url": "not-a-valid-url"}' | jq .
```

### Missing URL
```bash
curl -X POST http://localhost:3000/product/information \
  -H "Content-Type: application/json" \
  -d '{}' | jq .
```

### Invalid Session ID
```bash
curl -X POST http://localhost:3000/product/information \
  -H "Content-Type: application/json" \
  -d '{
    "url": "https://www.amazon.com/dp/B08N5WRWNW",
    "session_id": "invalid-session-id"
  }' | jq .
```

Expected error response:
```json
{
  "error": "Session invalid-session-id not found",
  "status": 404
}
```

## Session Management

### List Active Sessions
```bash
curl -X GET http://localhost:3000/health | jq '.active_sessions'
```

### Clean Up All Sessions
```bash
curl -X POST http://localhost:3000/browser/sessions/cleanup | jq .
```

### Clean Up Specific Session
```bash
curl -X DELETE http://localhost:3000/browser/session/$SESSION_ID | jq .
```

## Performance Tips

1. **Reuse Sessions**: Create a session once and reuse it for multiple extractions
2. **Clean Up**: Always clean up sessions when done to free resources
3. **Batch Requests**: Group multiple extractions together when possible
4. **Monitor**: Use the health endpoint to monitor server status

## Automation Example

Here's a complete example that creates a session, extracts multiple products, and cleans up:

```bash
#!/bin/bash

# Create session
SESSION_ID=$(curl -s -X POST http://localhost:3000/browser/session | jq -r '.session_id')
echo "Created session: $SESSION_ID"

# Extract multiple products
URLS=(
  "https://www.amazon.com/dp/B08N5WRWNW"
  "https://www.amazon.com/dp/B08KTZ8249"
  "https://www.amazon.com/Star-Wars-Echo-Dot-bundle/dp/B0DZQ92XQZ/"
)

for url in "${URLS[@]}"; do
  echo "Extracting: $url"
  curl -s -X POST http://localhost:3000/product/information \
    -H "Content-Type: application/json" \
    -d "{\"url\": \"$url\", \"session_id\": \"$SESSION_ID\"}" | \
    jq '.product | {name, price, availability}'
  echo "---"
done

# Clean up session
curl -s -X DELETE http://localhost:3000/browser/session/$SESSION_ID
echo "Session cleaned up"
``` 
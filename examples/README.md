# Examples

This directory contains example scripts and usage demonstrations for the LLM Web Agent.

## Quick Start

For a quick test of the product extraction functionality:

```bash
# Start the server
cargo run --release &

# Run the simple demo
./examples/simple_demo.sh
```

## Product Extraction Demo

### `simple_demo.sh`

A lightweight script that quickly tests the product extraction endpoint functionality.

**Features:**
- Server status verification
- Basic product extraction test
- Quick validation of API functionality
- Minimal dependencies (just `jq`)

**Usage:**
```bash
./examples/simple_demo.sh
```

### `demo_product_extraction.sh`

A comprehensive demonstration script that showcases the product information extraction capabilities of the LLM Web Agent.

#### Features Demonstrated

- **Automatic Session Management**: Shows both temporary and persistent session usage
- **Real-world Product Extraction**: Tests with actual Amazon product pages
- **Structured Output**: Displays extracted product information in a formatted way
- **Error Handling**: Demonstrates proper error handling and debugging
- **Performance Metrics**: Shows extraction timing and performance data

#### Prerequisites

1. **Server Running**: The LLM Web Agent server must be running on `localhost:3000`
2. **Dependencies**: `jq` must be installed for JSON parsing
3. **Network Access**: Internet connection required for fetching product pages

#### Quick Start

```bash
# Make sure the server is running
cargo run --release &

# Run the demo
./examples/demo_product_extraction.sh
```

#### Usage Options

```bash
# Run full demonstration
./examples/demo_product_extraction.sh

# Check server status only
./examples/demo_product_extraction.sh --check

# Show help
./examples/demo_product_extraction.sh --help
```

#### Example Output

The script will demonstrate:

1. **Scenario 1**: Extract Amazon Star Wars Echo Dot product information using a temporary session
2. **Scenario 2**: Create a persistent browser session and reuse it for multiple extractions

Sample output:
```
📦 EXTRACTED PRODUCT INFORMATION:
▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔
Name:         Amazon Echo Dot (5th Gen, 2022 release) | Star Wars Mandalorian Bundle
Description:  Smart speaker with Alexa and premium audio
Price:        $79.98
Availability: In Stock
Brand:        Amazon
Rating:       4.3 out of 5 stars
Image URL:    https://m.media-amazon.com/images/I/61H3K9GkbKL._AC_SL1000_.jpg
▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔
```

#### API Usage Examples

The script also provides direct API usage examples:

```bash
# Extract without session (creates temporary session)
curl -X POST http://localhost:3000/product/information \
  -H "Content-Type: application/json" \
  -d '{"url": "https://www.amazon.com/dp/B08N5WRWNW"}'

# Extract with existing session
curl -X POST http://localhost:3000/product/information \
  -H "Content-Type: application/json" \
  -d '{"url": "https://www.amazon.com/dp/B08N5WRWNW", "session_id": "your-session-id"}'
```

#### Installation Dependencies

**macOS (with Homebrew):**
```bash
brew install jq
```

**Ubuntu/Debian:**
```bash
sudo apt-get install jq
```

**CentOS/RHEL:**
```bash
sudo yum install jq
```

#### Troubleshooting

**Server Not Running:**
```bash
# Start the server
cargo build --release
./target/release/llm-web-agent &

# Or run in foreground
cargo run --release
```

**jq Not Installed:**
```bash
# The script will show installation instructions if jq is missing
./examples/demo_product_extraction.sh
```

**Browser Issues:**
If you encounter browser automation issues, ensure you have the necessary dependencies for headless Chrome/Chromium installed on your system.

#### Extending the Demo

You can easily modify the script to test additional URLs by adding them to the demonstration scenarios in the `main()` function.

Example URLs to try:
- Amazon products: `https://www.amazon.com/dp/PRODUCT_ID`
- Other e-commerce sites with product pages
- Any webpage with structured product information

## Additional Resources

### `curl_examples.md`

Comprehensive collection of cURL commands for direct API usage, including:
- Basic product extraction examples
- Session management
- Error handling scenarios
- Performance optimization tips
- Complete automation scripts

Perfect for developers who want to integrate the API into their own applications or scripts. 
# Load Testing with Drill

This directory contains load testing configurations for the LLM Web Agent using [Drill](https://github.com/fcsonline/drill), a powerful Rust-based HTTP load testing tool. All load testing commands are available through the project's Makefile.

## Prerequisites

1. **Drill CLI Tool**: Already installed in your system at `/Users/tlapiana/.cargo/bin/drill`
2. **Running Server**: Your LLM Web Agent server must be running on `http://localhost:3000`

## Quick Start

### 1. Start Your Server
```bash
# In one terminal, start the LLM Web Agent server
cargo run
```

### 2. Run Load Tests
```bash
# In another terminal, run the load tests

# Check if server is healthy
make load-test-check

# Run light load test (recommended for development)
make load-test-light

# Run standard load test
make load-test-standard

# Run heavy stress test (use with caution)
make load-test-heavy

# Run all standard tests (light + standard)
make load-test-all

# Clean up test reports
make load-test-clean
```

## Available Test Configurations

### 1. Light Load Test (`drill-light.yml`)
- **Purpose**: Basic functionality testing
- **Load**: 50 iterations, 5 concurrent users
- **Duration**: ~10 seconds ramp-up
- **Tests**: Health checks and session creation
- **Use Case**: Development and quick validation

### 2. Standard Load Test (`drill-config.yml`)
- **Purpose**: Comprehensive API testing
- **Load**: 100 iterations, 10 concurrent users  
- **Duration**: ~30 seconds ramp-up
- **Tests**: All API endpoints with realistic scenarios
- **Use Case**: Regular performance testing

### 3. Heavy Load Test (`drill-heavy.yml`)
- **Purpose**: Stress testing and bottleneck identification
- **Load**: 500 iterations, 50 concurrent users
- **Duration**: ~60 seconds ramp-up
- **Tests**: High-load scenarios with multiple sessions
- **Use Case**: Performance validation before production

## Test Scenarios Covered

### Core Functionality
- ✅ Health check endpoints (`/health`, `/`)
- ✅ Browser session management (`/browser/session`)
- ✅ Navigation requests (`/browser/navigate`)
- ✅ Browser interactions (`/browser/interact`)
- ✅ Data extraction (`/browser/extract`)
- ✅ AI automation tasks (`/automation/task`)

### Browser Actions Tested
- ✅ Click interactions
- ✅ Text input (typing)
- ✅ Wait operations
- ✅ Screenshot capture
- ✅ Page scrolling
- ✅ Element waiting

### Load Patterns
- ✅ Gradual ramp-up to simulate realistic traffic
- ✅ Random delays between requests (1-5 seconds)
- ✅ Concurrent user simulation
- ✅ Session state management

## Understanding Results

### Console Output
- Real-time statistics during test execution
- Response times, error rates, and throughput
- Color-coded status indicators

### HTML Reports
- Detailed visual reports with charts and graphs
- Response time distributions
- Error analysis and trends
- Generated files:
  - `drill-light-report.html`
  - `drill-report.html` 
  - `drill-heavy-report.html`

### JSON Reports
- Machine-readable test results
- Integration with CI/CD pipelines
- Detailed metrics for analysis

## Performance Metrics to Monitor

### Response Times
- **Target**: < 200ms for health checks
- **Target**: < 500ms for session operations
- **Target**: < 2s for browser interactions
- **Target**: < 5s for AI automation tasks

### Error Rates
- **Target**: < 1% error rate under normal load
- **Target**: < 5% error rate under stress conditions

### Throughput
- **Target**: > 50 requests/second for health endpoints
- **Target**: > 20 requests/second for browser operations

## Troubleshooting

### Common Issues

1. **Server Not Running**
   ```
   [ERROR] Server is not running or not healthy!
   ```
   **Solution**: Start the server with `cargo run`

2. **High Error Rates**
   - Check server logs for errors
   - Reduce concurrency in test config
   - Ensure sufficient system resources

3. **Timeouts**
   - Increase timeout values in drill configs
   - Check network connectivity
   - Monitor system resource usage

### System Requirements

- **CPU**: Multi-core recommended for concurrent testing
- **Memory**: 2GB+ available RAM
- **Network**: Stable local network connection
- **Browser**: Chrome/Chromium for browser automation

## Customization

### Modifying Test Parameters

Edit the YAML configuration files to adjust:
- `iterations`: Total number of test iterations
- `concurrency`: Number of concurrent virtual users
- `rampup`: Time to gradually increase load (seconds)
- `delay`: Random delay between requests

### Adding New Test Scenarios

1. Add new test steps to the `plan` section in YAML files
2. Include appropriate request headers and body content
3. Follow the existing pattern for consistent testing

### Example Custom Test
```yaml
- name: custom_endpoint
  request:
    url: /your/custom/endpoint
    method: POST
    headers:
      Content-Type: 'application/json'
    body: |
      {
        "custom_data": "test_value"
      }
```

## Integration with CI/CD

The load tests can be integrated into your CI/CD pipeline:

```bash
# Example CI script
make load-test-check || exit 1
make load-test-light || exit 1
make load-test-clean
```

## Best Practices

1. **Start Small**: Begin with light tests before running heavy loads
2. **Monitor Resources**: Watch CPU, memory, and network during tests
3. **Baseline Measurements**: Establish performance baselines for comparison
4. **Regular Testing**: Include load testing in your development workflow
5. **Environment Consistency**: Use consistent test environments
6. **Clean Up**: Remove test reports after analysis

## Support

For issues with:
- **Drill Tool**: Check [Drill GitHub repository](https://github.com/fcsonline/drill)
- **LLM Web Agent**: Check application logs and server status
- **Load Test Configs**: Modify YAML files based on your needs 
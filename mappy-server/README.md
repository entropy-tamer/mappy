# Mappy Server - Network Server for Mappy Service

A high-performance HTTP server that provides network access to the Mappy Engine. Built with Axum and designed for production deployment with comprehensive error handling, monitoring, and real-time error tracking.

## 🚀 Features

- **🌐 HTTP API**: RESTful endpoints for Engine operations (get/set)
- **⚡ High Performance**: Built with Axum for maximum throughput
- **🔐 Security**: TLS support and authentication ready
- **📊 Monitoring**: Built-in health checks, metrics, and error tracking
- **🔧 Configuration**: Environment-based configuration management
- **📝 Logging**: Comprehensive structured logging with tracing
- **🗺️ Engine Support**: Full support for Mappy Engine with error tracking and one-sided error guarantee monitoring
- **📡 Real-time Updates**: Server-Sent Events (SSE) for streaming error updates

## 📦 Installation

### From Source

```bash
# Clone the repository
git clone https://github.com/entropy-tamer/mappy.git
cd mappy

# Build the server
cargo build --release --bin mappy-server

# Run the server
./target/release/mappy-server
```

### Docker

```bash
# Build Docker image
docker build -t mappy-server .

# Run container
docker run -p 8080:8080 mappy-server
```

## 🚀 Quick Start

### Basic Usage

```bash
# Start the server with default configuration
./mappy-server

# Start with custom configuration
MAPPY_CAPACITY=10000 MAPPY_FALSE_POSITIVE_RATE=0.01 ./mappy-server

# Start with custom port
MAPPY_PORT=3000 ./mappy-server
```

### Environment Variables

```bash
# Server configuration (Unix socket is default)
MAPPY_SOCKET_PATH="/var/run/reynard/mappy.sock"  # Unix socket path (default)
MAPPY_ENABLE_SOCKET="true"                     # Enable Unix socket server (default: true)
MAPPY_ENABLE_HTTP="false"                     # Enable HTTP server (default: false)
MAPPY_PORT=8003                               # HTTP port (only used if ENABLE_HTTP=true)
MAPPY_HOST=0.0.0.0                            # HTTP host (only used if ENABLE_HTTP=true)

# Engine configuration
MAPPY_CAPACITY=10000
MAPPY_FALSE_POSITIVE_RATE=0.01

# Storage configuration
MAPPY_DATA_DIR="./data/mappy"
MAPPY_PERSISTENCE_MODE="memory"

# Log level
RUST_LOG="mappy=info"
```

### Communication Methods

Mappy Server supports two communication methods:

1. **Unix Domain Sockets (Default)**: Fast, secure local communication
   - Socket path: `/var/run/reynard/mappy.sock` (configurable via `MAPPY_SOCKET_PATH`)
   - No network overhead, better performance
   - Only accessible from the same machine

2. **HTTP Server (Optional)**: Network-accessible API
   - Enabled via `MAPPY_ENABLE_HTTP=true`
   - Listens on `MAPPY_HOST:MAPPY_PORT` (default: `0.0.0.0:8003`)
   - Can run simultaneously with Unix socket

**Examples:**

```bash
# Unix socket only (default)
./mappy-server

# HTTP only
MAPPY_ENABLE_HTTP=true MAPPY_PORT=8003 ./mappy-server

# Both socket and HTTP
MAPPY_ENABLE_HTTP=true MAPPY_SOCKET_PATH=/var/run/reynard/mappy.sock MAPPY_PORT=8003 ./mappy-server
```

## 🔧 Configuration

### Configuration File

Create a `config.toml` file:

```toml
[server]
port = 8080
host = "0.0.0.0"

[maplet]
capacity = 10000
false_positive_rate = 0.01

[storage]
data_dir = "./data/mappy"
persistence_mode = "hybrid"

[logging]
level = "info"
format = "json"
```

### Configuration Options

| Option                       | Type   | Default          | Description                              |
| ---------------------------- | ------ | ---------------- | ---------------------------------------- |
| `server.port`                | u16    | `8080`           | Server port                              |
| `server.host`                | String | `"0.0.0.0"`      | Server host                              |
| `maplet.capacity`            | usize  | `10000`          | Initial maplet capacity                  |
| `maplet.false_positive_rate` | f64    | `0.01`           | False positive rate (1%)                 |
| `storage.data_dir`           | String | `"./data/mappy"` | Directory for persistent storage         |
| `storage.persistence_mode`   | String | `"hybrid"`       | Storage mode: `memory`, `disk`, `hybrid` |
| `logging.level`              | String | `"info"`         | Log level                                |
| `logging.format`             | String | `"pretty"`       | Log format: `pretty`, `json`             |

## 📚 API Reference

### Health Check

```http
GET /health
```

Returns server health status.

**Response:**

```json
{
  "status": "healthy"
}
```

### Status

```http
GET /status
```

Returns server and engine status.

**Response:**

```json
{
  "status": "running",
  "engine_ready": true
}
```

### Set Key-Value Pair

```http
POST /set
Content-Type: application/json

{
  "key": "user:123",
  "value": "some_value"
}
```

**Response:** `200 OK` on success, `500 Internal Server Error` on failure, `503 Service Unavailable` if engine is not ready.

### Get Value

```http
GET /get/{key}
```

**Response:**

```json
{
  "key": "user:123",
  "value": "some_value",
  "found": true
}
```

If the key is not found:

```json
{
  "key": "user:123",
  "value": null,
  "found": false
}
```

### Error Tracking

#### Get Errors

```http
GET /api/errors?limit=100&error_type=CapacityExceeded&severity=High&since=1234567890
```

Query parameters:
- `limit`: Maximum number of errors to return (default: 100, max: 1000)
- `error_type`: Filter by error type (CapacityExceeded, CollisionLimitExceeded, ResizeFailed, SerializationError, Internal, OneSidedError)
- `severity`: Filter by severity (Low, Medium, High, Critical)
- `since`: Only return errors after this timestamp (Unix seconds)

**Response:**

```json
{
  "total_count": 150,
  "filtered_count": 25,
  "errors": [
    {
      "error_type": "CapacityExceeded",
      "timestamp": 1234567890,
      "key": "user:123",
      "severity": "High"
    }
  ]
}
```

#### Clear Errors

```http
DELETE /api/errors
```

Clears all logged errors.

**Response:** `200 OK`

#### Get Error Statistics

```http
GET /api/errors/stats
```

Returns aggregated error statistics.

**Response:**

```json
{
  "total_errors": 150,
  "error_rate": 0.015,
  "by_type": {
    "CapacityExceeded": 10,
    "CollisionLimitExceeded": 5,
    "ResizeFailed": 2,
    "SerializationError": 3,
    "Internal": 1,
    "OneSidedError": 129
  },
  "by_severity": {
    "Low": 130,
    "Medium": 5,
    "High": 12,
    "Critical": 3
  },
  "collision_count": 5,
  "max_chain_length": 8
}
```

#### Get One-Sided Errors

```http
GET /api/errors/one-sided
```

Returns errors related to one-sided error guarantees.

**Response:**

```json
{
  "total_count": 129,
  "errors": [
    {
      "error_type": "OneSidedError",
      "timestamp": 1234567890,
      "key": "user:123",
      "expected_value": "42",
      "actual_value": "45",
      "severity": "Low"
    }
  ]
}
```

### Metrics

```http
GET /api/metrics
```

Returns system metrics and performance data.

**Response:**

```json
{
  "uptime_seconds": 3600,
  "requests_total": 5000,
  "requests_per_second": 1.39,
  "error_rate": 0.002
}
```

### Error Streaming

```http
GET /api/errors/stream
```

Server-Sent Events (SSE) stream for real-time error updates.

**Response:** SSE event stream with error events.

## 🔐 Security

- **TLS Support**: HTTPS with configurable certificates
- **Authentication**: Ready for JWT or API key authentication
- **Input Validation**: Comprehensive request validation
- **Rate Limiting**: Built-in protection against abuse
- **CORS**: Configurable cross-origin resource sharing

## 📊 Monitoring

### Health Endpoints

- `GET /health` - Basic health check
- `GET /status` - Server and engine status
- `GET /api/metrics` - System metrics and performance data

### Error Tracking Endpoints

- `GET /api/errors` - Query error log with filtering
- `DELETE /api/errors` - Clear error log
- `GET /api/errors/stats` - Aggregated error statistics
- `GET /api/errors/one-sided` - One-sided error guarantees tracking
- `GET /api/errors/stream` - Real-time error streaming via SSE

### Metrics

- Request count and duration
- Error rates by type and severity
- Collision tracking and chain length monitoring
- One-sided error guarantee violations
- System uptime and requests per second

## 🧪 Testing

```bash
# Run server tests
cargo test --bin mappy-server

# Run integration tests
cargo test --test integration

# Run with specific log level
RUST_LOG=debug cargo test --bin mappy-server
```

## 🚀 Deployment

### Production Checklist

- [ ] Configure appropriate `MAPPY_CAPACITY`
- [ ] Set optimal `MAPPY_FALSE_POSITIVE_RATE`
- [ ] Configure proper `MAPPY_DATA_DIR`
- [ ] Set up TLS certificates
- [ ] Configure logging level
- [ ] Set up monitoring and alerting
- [ ] Configure backup strategy
- [ ] Set up load balancing (if needed)

### Docker Deployment

```dockerfile
FROM rust:1.75 as builder
WORKDIR /app
COPY . .
RUN cargo build --release --bin mappy-server

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/mappy-server /usr/local/bin/
EXPOSE 8080
CMD ["mappy-server"]
```

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](../../LICENSE) file for details.

## 🙏 Acknowledgments

- Built with [Axum](https://github.com/tokio-rs/axum) for high-performance HTTP
- Uses [Tower](https://github.com/tower-rs/tower) for middleware
- Logging powered by [Tracing](https://github.com/tokio-rs/tracing)
- Configuration managed by [Config](https://github.com/mehcode/config-rs)
- Core maplet functionality from [mappy-core](../mappy-core/)

## 📞 Support

- 📖 [Documentation](https://github.com/entropy-tamer/mappy/wiki)
- 🐛 [Issue Tracker](https://github.com/entropy-tamer/mappy/issues)
- 💬 [Discussions](https://github.com/entropy-tamer/mappy/discussions)

---

## Made with ❤️ by the Reynard Team

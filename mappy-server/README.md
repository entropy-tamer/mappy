# Mappy Server - Network Server for Mappy Service

A high-performance HTTP server that provides network access to the Mappy service. Built with Axum and designed for production deployment with comprehensive error handling and monitoring.

## 🚀 Features

- **🌐 HTTP API**: RESTful endpoints for all Mappy operations
- **⚡ High Performance**: Built with Axum for maximum throughput
- **🔐 Security**: TLS support and authentication ready
- **📊 Monitoring**: Built-in health checks and metrics
- **🔧 Configuration**: Environment-based configuration management
- **📝 Logging**: Comprehensive structured logging with tracing
- **🗺️ Maplet Support**: Full support for space-efficient maplet operations

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
# Server configuration
MAPPY_PORT=8080
MAPPY_HOST=0.0.0.0

# Maplet configuration
MAPPY_CAPACITY=10000
MAPPY_FALSE_POSITIVE_RATE=0.01

# Storage configuration
MAPPY_DATA_DIR="./data/mappy"
MAPPY_PERSISTENCE_MODE="hybrid"

# Log level
RUST_LOG="mappy=info"
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
  "status": "healthy",
  "timestamp": "2025-01-15T10:30:00Z",
  "version": "0.1.0"
}
```

### Maplet Operations

#### Insert Key-Value Pair

```http
POST /api/v1/maplet/insert
Content-Type: application/json

{
  "key": "user:123",
  "value": 42
}
```

#### Query Value

```http
GET /api/v1/maplet/query/{key}
```

**Response:**

```json
{
  "key": "user:123",
  "value": 42,
  "found": true
}
```

#### Check Key Exists

```http
HEAD /api/v1/maplet/contains/{key}
```

#### Delete Key

```http
DELETE /api/v1/maplet/delete/{key}
```

#### Get Statistics

```http
GET /api/v1/maplet/stats
```

**Response:**

```json
{
  "capacity": 10000,
  "size": 1500,
  "load_factor": 0.15,
  "memory_usage": 1024000,
  "false_positive_rate": 0.01
}
```

### Advanced Operations

#### Find Slot for Key

```http
GET /api/v1/maplet/slot/{key}
```

**Response:**

```json
{
  "key": "user:123",
  "slot": 42,
  "found": true
}
```

#### Batch Operations

```http
POST /api/v1/maplet/batch/insert
Content-Type: application/json

{
  "operations": [
    {"key": "user:1", "value": 10},
    {"key": "user:2", "value": 20},
    {"key": "user:3", "value": 30}
  ]
}
```

## 🔐 Security

- **TLS Support**: HTTPS with configurable certificates
- **Authentication**: Ready for JWT or API key authentication
- **Input Validation**: Comprehensive request validation
- **Rate Limiting**: Built-in protection against abuse
- **CORS**: Configurable cross-origin resource sharing

## 📊 Monitoring

### Health Endpoints

- `GET /health` - Basic health check
- `GET /health/detailed` - Detailed system status
- `GET /metrics` - Prometheus-compatible metrics

### Metrics

- Request count and duration
- Error rates by endpoint
- Memory usage and maplet statistics
- False positive rate monitoring

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

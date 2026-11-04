# 🚀 Deployment Guide

This guide covers deploying your RustyX application to production.

## Table of Contents

- [Production Build](#production-build)
- [Environment Configuration](#environment-configuration)
- [Docker Deployment](#docker-deployment)
- [Linux Server Deployment](#linux-server-deployment)
- [Cloud Platforms](#cloud-platforms)
- [Reverse Proxy Setup](#reverse-proxy-setup)
- [SSL/TLS Configuration](#ssltls-configuration)
- [Monitoring & Logging](#monitoring--logging)
- [Performance Optimization](#performance-optimization)

---

## Production Build

### Create Optimized Binary

```bash
# Standard release build
cargo build --release

# With link-time optimization (slower build, faster binary)
RUSTFLAGS="-C target-cpu=native" cargo build --release
```

The binary is at `./target/release/your_app`.

### Cargo.toml Optimizations

```toml
[profile.release]
opt-level = 3
lto = true
codegen-units = 1
panic = "abort"
strip = true
```

---

## Environment Configuration

### Using Environment Variables

```rust
use std::env;

fn main() {
    // Load .env file in development
    #[cfg(debug_assertions)]
    dotenv::dotenv().ok();

    let config = AppConfig {
        port: env::var("PORT")
            .unwrap_or_else(|_| "3000".to_string())
            .parse()
            .expect("PORT must be a number"),
        database_url: env::var("DATABASE_URL")
            .expect("DATABASE_URL must be set"),
        jwt_secret: env::var("JWT_SECRET")
            .expect("JWT_SECRET must be set"),
        environment: env::var("RUST_ENV")
            .unwrap_or_else(|_| "development".to_string()),
    };
}
```

### Example .env File

```env
# Server
PORT=3000
HOST=0.0.0.0
RUST_LOG=info

# Database
DATABASE_URL=postgres://user:password@localhost:5432/mydb

# Security
JWT_SECRET=your-super-secret-key-change-in-production
API_KEY=your-api-key

# External Services
REDIS_URL=redis://localhost:6379
SMTP_HOST=smtp.example.com
```

---

## Docker Deployment

### Dockerfile

```dockerfile
# Build stage
FROM rust:1.75-slim as builder

WORKDIR /app

# Install dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Create dummy main.rs and build dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release
RUN rm -rf src

# Copy source code
COPY src ./src

# Build application
RUN touch src/main.rs && cargo build --release

# Runtime stage
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -m -u 1000 app
USER app

WORKDIR /app

# Copy binary
COPY --from=builder /app/target/release/your_app /app/

# Expose port
EXPOSE 3000

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:3000/health || exit 1

# Run
CMD ["./your_app"]
```

### Docker Compose

```yaml
version: '3.8'

services:
  api:
    build: .
    ports:
      - "3000:3000"
    environment:
      - RUST_LOG=info
      - DATABASE_URL=postgres://postgres:password@db:5432/mydb
      - REDIS_URL=redis://redis:6379
    depends_on:
      db:
        condition: service_healthy
      redis:
        condition: service_started
    restart: unless-stopped
    networks:
      - app-network

  db:
    image: postgres:15-alpine
    environment:
      POSTGRES_USER: postgres
      POSTGRES_PASSWORD: password
      POSTGRES_DB: mydb
    volumes:
      - postgres_data:/var/lib/postgresql/data
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U postgres"]
      interval: 5s
      timeout: 5s
      retries: 5
    networks:
      - app-network

  redis:
    image: redis:7-alpine
    volumes:
      - redis_data:/data
    networks:
      - app-network

  nginx:
    image: nginx:alpine
    ports:
      - "80:80"
      - "443:443"
    volumes:
      - ./nginx.conf:/etc/nginx/nginx.conf:ro
      - ./ssl:/etc/nginx/ssl:ro
    depends_on:
      - api
    networks:
      - app-network

volumes:
  postgres_data:
  redis_data:

networks:
  app-network:
    driver: bridge
```

### Build and Run

```bash
# Build
docker build -t my-api .

# Run single container
docker run -d -p 3000:3000 --name my-api \
  -e DATABASE_URL=postgres://... \
  my-api

# Docker Compose
docker-compose up -d

# View logs
docker-compose logs -f api

# Scale
docker-compose up -d --scale api=3
```

---

## Linux Server Deployment

### Systemd Service

Create `/etc/systemd/system/rustyx-api.service`:

```ini
[Unit]
Description=RustyX API Server
After=network.target postgresql.service
Wants=network-online.target

[Service]
Type=simple
User=www-data
Group=www-data
WorkingDirectory=/opt/my-api
ExecStart=/opt/my-api/my_app
Restart=always
RestartSec=5
StartLimitBurst=3
StartLimitInterval=60

# Environment
Environment=RUST_LOG=info
Environment=PORT=3000
EnvironmentFile=/opt/my-api/.env

# Security
NoNewPrivileges=yes
ProtectSystem=strict
ProtectHome=yes
PrivateTmp=yes
ReadWritePaths=/var/log/my-api

# Resource limits
LimitNOFILE=65535
MemoryMax=512M

[Install]
WantedBy=multi-user.target
```

### Deploy Script

```bash
#!/bin/bash
set -e

APP_NAME="my-api"
DEPLOY_DIR="/opt/$APP_NAME"
BINARY_NAME="my_app"

echo "🚀 Deploying $APP_NAME..."

# Build
cargo build --release

# Stop service
sudo systemctl stop $APP_NAME || true

# Copy binary
sudo cp target/release/$BINARY_NAME $DEPLOY_DIR/

# Set permissions
sudo chown www-data:www-data $DEPLOY_DIR/$BINARY_NAME
sudo chmod 755 $DEPLOY_DIR/$BINARY_NAME

# Restart service
sudo systemctl start $APP_NAME
sudo systemctl status $APP_NAME

echo "✅ Deployment complete!"
```

### Commands

```bash
# Enable service
sudo systemctl enable rustyx-api

# Start
sudo systemctl start rustyx-api

# Stop
sudo systemctl stop rustyx-api

# Restart
sudo systemctl restart rustyx-api

# View logs
sudo journalctl -u rustyx-api -f

# Check status
sudo systemctl status rustyx-api
```

---

## Cloud Platforms

### Railway

```bash
# Install CLI
npm install -g @railway/cli

# Login
railway login

# Initialize
railway init

# Deploy
railway up
```

### Fly.io

```bash
# Install CLI
curl -L https://fly.io/install.sh | sh

# Login
fly auth login

# Initialize
fly launch

# Deploy
fly deploy

# Scale
fly scale count 2
```

### Render

1. Connect GitHub repository
2. Select "Web Service"
3. Configure:
   - Build Command: `cargo build --release`
   - Start Command: `./target/release/my_app`
4. Add environment variables
5. Deploy

### AWS EC2

```bash
# Connect to instance
ssh -i your-key.pem ec2-user@your-instance

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Clone and build
git clone https://github.com/you/your-app.git
cd your-app
cargo build --release

# Configure systemd (see above)
```

---

## Reverse Proxy Setup

### Nginx Configuration

```nginx
# /etc/nginx/sites-available/api.example.com

upstream rustyx_backend {
    server 127.0.0.1:3000;
    server 127.0.0.1:3001;  # If running multiple instances
    keepalive 32;
}

server {
    listen 80;
    server_name api.example.com;
    return 301 https://$server_name$request_uri;
}

server {
    listen 443 ssl http2;
    server_name api.example.com;

    # SSL
    ssl_certificate /etc/letsencrypt/live/api.example.com/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/api.example.com/privkey.pem;
    ssl_protocols TLSv1.2 TLSv1.3;
    ssl_ciphers ECDHE-ECDSA-AES128-GCM-SHA256:ECDHE-RSA-AES128-GCM-SHA256;
    ssl_prefer_server_ciphers off;

    # Security headers
    add_header X-Frame-Options "SAMEORIGIN" always;
    add_header X-Content-Type-Options "nosniff" always;
    add_header X-XSS-Protection "1; mode=block" always;

    # Gzip
    gzip on;
    gzip_types application/json text/plain application/javascript;

    location / {
        proxy_pass http://rustyx_backend;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection 'upgrade';
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        proxy_cache_bypass $http_upgrade;
        proxy_read_timeout 300;
        proxy_connect_timeout 300;
    }

    # Health check endpoint
    location /health {
        proxy_pass http://rustyx_backend/health;
        access_log off;
    }
}
```

### Enable Site

```bash
sudo ln -s /etc/nginx/sites-available/api.example.com /etc/nginx/sites-enabled/
sudo nginx -t
sudo systemctl reload nginx
```

---

## SSL/TLS Configuration

### Let's Encrypt with Certbot

```bash
# Install
sudo apt install certbot python3-certbot-nginx

# Get certificate
sudo certbot --nginx -d api.example.com

# Auto-renewal
sudo certbot renew --dry-run
```

### Cron for Auto-renewal

```bash
# Add to /etc/cron.d/certbot
0 0,12 * * * root certbot renew --quiet --post-hook "systemctl reload nginx"
```

---

## Monitoring & Logging

### Structured Logging

```rust
use tracing::{info, error, warn, instrument};

#[instrument]
async fn process_request(req: Request) -> Response {
    info!("Processing request");
    // ...
}
```

### Prometheus Metrics

Add health and metrics endpoints:

```rust
app.get("/health", |_req, res| async move {
    res.json(json!({
        "status": "healthy",
        "uptime": get_uptime(),
        "version": env!("CARGO_PKG_VERSION")
    }))
});

app.get("/metrics", |_req, res| async move {
    // Prometheus format
    res.content_type("text/plain").send(format!(
        "http_requests_total {{}} {}\n\
         http_request_duration_seconds {{}} {}",
        get_request_count(),
        get_avg_duration()
    ))
});
```

---

## Performance Optimization

### Connection Pooling

```rust
// Use connection pooling for databases
let pool = PgPoolOptions::new()
    .max_connections(20)
    .min_connections(5)
    .acquire_timeout(Duration::from_secs(5))
    .connect(&database_url)
    .await?;
```

### Response Caching

```rust
app.use_middleware(|req, res, next| async move {
    // Cache GET requests
    if req.method() == &hyper::Method::GET {
        let response = next(req, res).await;
        response.header("cache-control", "public, max-age=300")
    } else {
        next(req, res).await
    }
});
```

### Load Balancing

Run multiple instances with different ports:

```bash
PORT=3000 ./my_app &
PORT=3001 ./my_app &
PORT=3002 ./my_app &
```

Configure Nginx upstream to load balance.

---

## Checklist

- [ ] Environment variables configured
- [ ] Production build with optimizations
- [ ] Database connection pooling
- [ ] Rate limiting enabled
- [ ] CORS properly configured
- [ ] SSL/TLS certificate installed
- [ ] Health check endpoint working
- [ ] Logging to file/service
- [ ] Monitoring set up
- [ ] Backups configured
- [ ] Auto-restart on failure
- [ ] Load balancing (if needed)

---

## Next Steps

- [API Reference](./API.md)
- [Troubleshooting Guide](./TROUBLESHOOTING.md)

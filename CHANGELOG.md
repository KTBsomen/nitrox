# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.0.1] - 2026-11-04

### Added
-  Initial release of **Nitrox** — A fast, minimalist web framework for Rust inspired by ExpressJS.
-  High-performance asynchronous HTTP engine built on Hyper 1.x and Tokio.
-  ExpressJS-like intuitive routing (`GET`, `POST`, `PUT`, `DELETE`, `PATCH`) with fast route matching via `matchit`.
-  Comprehensive middleware pipeline:
  - `logger()` - Request logging with tracing
  - `cors()` & `cors_with_options()` - Configurable CORS headers
  - `helmet()` - Essential HTTP security headers
  - `timeout()` - Request timeouts
  - `rate_limiter()` & `simple_rate_limit()` - Abuse and DDoS prevention
  - `request_id()` - Automatic `X-Request-ID` injection
  - `response_time()` - Response latency tracking
-  **File Upload Module**:
  - Multipart form handling with streaming boundary parser
  - File validation (MIME types, extensions, size limits)
  - Disk and memory storage options with UUID and sanitized naming
-  **WebSocket Support**:
  - Real-time bidirectional communication
  - Room-based messaging and broadcast capability
-  **Static File Serving**:
  - Automatic MIME type detection, cache control headers, and index file serving
-  Fully database-agnostic architecture: cleanly integrate SQLx, Diesel, SeaORM, MongoDB, or Redis in application state without unnecessary bundled dependencies.
-  Type-safe JSON handling and structured error propagation with `thiserror 2.0`.

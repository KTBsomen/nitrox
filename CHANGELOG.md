# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Planning future features

## [0.2.0] - 2024-12-30

### Added
- 📤 **File Upload Module** - Multer-like file upload functionality
  - Single and multiple file uploads
  - File type validation (MIME types & extensions)
  - File size limits
  - Disk and memory storage options
  - UUID, timestamp, or custom file naming
  - Preset configurations (images_only, documents_only)
- 🌐 **WebSocket Support** - Real-time bidirectional communication
  - Connection management
  - Room-based messaging
  - Broadcast functionality
- ⏱️ **Rate Limiting Middleware** - Protect APIs from abuse
  - Configurable request limits
  - Time window settings
  - Skip paths option
- 📁 **Static File Serving** - Serve static assets
  - MIME type detection
  - Cache control headers
  - Index file support
- 🔧 **Additional Middleware**
  - `request_id()` - Add X-Request-ID header
  - `response_time()` - Add X-Response-Time header
  - `cors_with_options()` - Advanced CORS configuration
- 📚 **Comprehensive Documentation**
  - docs/INSTALLATION.md
  - docs/QUICKSTART.md
  - docs/MIDDLEWARE.md
  - docs/UPLOAD.md
  - docs/DEPLOYMENT.md
- 🧪 **File Upload Example** - Complete example with HTML form

### Changed
- Improved README with more examples
- Enhanced lib.rs documentation for docs.rs

## [0.1.0] - 2024-12-29

### Added
- 🎉 Initial release of RustyX
- ExpressJS-like routing (GET, POST, PUT, DELETE, PATCH)
- Request object with params, query, body parsing
- Response object with JSON, HTML, text support
- Middleware support with Next function
- Built-in middleware: logger, CORS, helmet, timeout
- Database module with multi-driver support
  - SQLite support (default)
  - MySQL support (feature flag)
  - PostgreSQL support (feature flag)
  - MongoDB support (feature flag)
- Query builder for SQL databases
- Model trait for database entities
- Controller trait for request handlers
- Route groups and API versioning helpers
- Utility functions (pagination, validation, etc.)
- Comprehensive documentation
- Example applications

### Security
- Helmet middleware for security headers
- CORS middleware for cross-origin requests
- Input validation helpers

---

## Version History

| Version | Date | Description |
|---------|------|-------------|
| 0.2.0 | 2024-12-30 | File Upload, WebSocket, Rate Limiting, Static Files |
| 0.1.0 | 2024-12-29 | Initial release |

[Unreleased]: https://github.com/Mohammad007/rustyx/compare/v0.2.0...HEAD
[0.2.0]: https://github.com/Mohammad007/rustyx/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/Mohammad007/rustyx/releases/tag/v0.1.0

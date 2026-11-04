# 📁 File Upload Guide

RustyX provides a powerful file upload module similar to Express's Multer.

## Table of Contents

- [Quick Start](#quick-start)
- [Configuration](#configuration)
- [Single File Upload](#single-file-upload)
- [Multiple File Upload](#multiple-file-upload)
- [File Validation](#file-validation)
- [Storage Options](#storage-options)
- [File Naming](#file-naming)
- [Error Handling](#error-handling)
- [Complete Example](#complete-example)

---

## Quick Start

```rust
use rustyx::prelude::*;

// Create an uploader
let uploader = Uploader::new(
    UploadConfig::new()
        .destination("./uploads")
        .max_file_size_mb(5)
        .allowed_extensions(vec!["png", "jpg", "pdf"])
);

// Use in route handler
app.post("/upload", move |req, res| {
    let uploader = uploader.clone();
    async move {
        // Parse multipart form data
        let content_type = req.content_type().unwrap_or_default();
        let boundary = parse_boundary(&content_type).unwrap();
        let fields = parse_multipart(req.body(), &boundary).unwrap();
        
        // Upload file
        for field in fields {
            if let Some(filename) = field.filename {
                let result = uploader.upload_single(
                    &field.name,
                    field.data,
                    &filename,
                    &field.content_type.unwrap_or_default()
                ).await;
                
                match result {
                    Ok(file) => return res.json(json!({ "file": file.filename })),
                    Err(e) => return res.bad_request(&e.to_string())
                }
            }
        }
        res.bad_request("No file provided")
    }
});
```

---

## Configuration

### UploadConfig Options

```rust
UploadConfig::new()
    // Storage
    .destination("./uploads")     // Upload directory
    .memory()                     // Use memory instead of disk
    
    // Size limits
    .max_file_size(5 * 1024 * 1024)  // 5MB in bytes
    .max_file_size_mb(5)              // 5MB (shorthand)
    .max_files(10)                    // Max files per request
    
    // Allowed types
    .allowed_types(vec![
        "image/png",
        "image/jpeg",
        "application/pdf"
    ])
    .allowed_extensions(vec!["png", "jpg", "pdf"])
    
    // Presets
    .images_only()      // PNG, JPG, JPEG, GIF, WebP, SVG
    .documents_only()   // PDF, DOC, DOCX, XLS, XLSX, TXT
    
    // Naming
    .keep_original_name()  // Use original filename
    .use_uuid()            // UUID with extension (default)
```

---

## Single File Upload

### Basic Upload

```rust
let uploader = Uploader::disk("./uploads");

app.post("/upload", move |req, res| {
    let uploader = uploader.clone();
    async move {
        let content_type = req.content_type().unwrap_or_default();
        let boundary = parse_boundary(&content_type).unwrap();
        let fields = parse_multipart(req.body(), &boundary)?;
        
        for field in fields {
            if field.name == "file" {
                if let Some(filename) = field.filename {
                    let file = uploader.upload_single(
                        "file",
                        field.data,
                        &filename,
                        &field.content_type.unwrap_or_default()
                    ).await?;
                    
                    return res.json(json!({
                        "filename": file.filename,
                        "size": file.size,
                        "path": file.path.to_string_lossy()
                    }));
                }
            }
        }
        res.bad_request("No file")
    }
});
```

### UploadedFile Properties

| Property | Type | Description |
|----------|------|-------------|
| `original_name` | `String` | Original filename from client |
| `filename` | `String` | Saved filename (may be UUID) |
| `path` | `PathBuf` | Full path to saved file |
| `mimetype` | `String` | MIME type |
| `size` | `usize` | File size in bytes |
| `field_name` | `String` | Form field name |
| `extension` | `String` | File extension |

---

## Multiple File Upload

```rust
let uploader = Uploader::new(
    UploadConfig::new()
        .destination("./uploads")
        .max_files(5)
        .images_only()
);

app.post("/upload-multiple", move |req, res| {
    let uploader = uploader.clone();
    async move {
        let boundary = parse_boundary(req.content_type().unwrap_or_default()).unwrap();
        let fields = parse_multipart(req.body(), &boundary)?;
        
        // Collect all files
        let mut files_data = Vec::new();
        for field in fields {
            if let Some(filename) = field.filename {
                files_data.push((
                    field.name,
                    field.data,
                    filename,
                    field.content_type.unwrap_or_default()
                ));
            }
        }
        
        let files = uploader.upload_multiple(files_data).await?;
        
        res.json(json!({
            "count": files.len(),
            "files": files.iter().map(|f| f.filename.clone()).collect::<Vec<_>>()
        }))
    }
});
```

---

## File Validation

### By MIME Type

```rust
let uploader = Uploader::new(
    UploadConfig::new()
        .allowed_types(vec![
            "image/png",
            "image/jpeg",
            "image/gif",
            "application/pdf"
        ])
);
```

### By Extension

```rust
let uploader = Uploader::new(
    UploadConfig::new()
        .allowed_extensions(vec!["png", "jpg", "jpeg", "pdf", "doc"])
);
```

### Presets

```rust
// Images only (PNG, JPG, JPEG, GIF, WebP, SVG)
.images_only()

// Documents only (PDF, DOC, DOCX, XLS, XLSX, TXT)
.documents_only()
```

### Size Limits

```rust
.max_file_size(10 * 1024 * 1024)  // 10MB in bytes
.max_file_size_mb(10)              // 10MB shorthand
```

---

## Storage Options

### Disk Storage (Default)

```rust
let uploader = Uploader::new(
    UploadConfig::new()
        .destination("./uploads")
);

// Or simply:
let uploader = Uploader::disk("./uploads");
```

### Memory Storage

```rust
let uploader = Uploader::new(
    UploadConfig::new()
        .memory()
);

// Or simply:
let uploader = Uploader::memory();
```

---

## File Naming

### UUID (Default)

Files are saved with UUID names: `550e8400-e29b-41d4-a716-446655440000.png`

```rust
.use_uuid()
```

### Original Name

Keep the original filename:

```rust
.keep_original_name()
```

### Timestamp

Use timestamp as filename:

```rust
.naming(FileNaming::TimestampWithExtension)
// Result: 1703894400000.png
```

### Custom Prefix

Use custom prefix with UUID:

```rust
.naming(FileNaming::CustomPrefix("avatar".to_string()))
// Result: avatar_550e8400-e29b-41d4-a716-446655440000.png
```

---

## Error Handling

### Upload Errors

```rust
pub enum UploadError {
    FileTooLarge { max: usize, actual: usize },
    TypeNotAllowed { mimetype: String },
    ExtensionNotAllowed { extension: String },
    TooManyFiles { max: usize, actual: usize },
    NoFile,
    FieldNotFound { field: String },
    IoError(String),
    ParseError(String),
}
```

### Handling Errors

```rust
match uploader.upload_single(...).await {
    Ok(file) => {
        res.json(json!({ "success": true, "file": file.filename }))
    }
    Err(UploadError::FileTooLarge { max, actual }) => {
        res.bad_request(&format!("File too large: {} bytes (max: {})", actual, max))
    }
    Err(UploadError::TypeNotAllowed { mimetype }) => {
        res.bad_request(&format!("File type not allowed: {}", mimetype))
    }
    Err(e) => {
        res.bad_request(&e.to_string())
    }
}
```

---

## Complete Example

```rust
use rustyx::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    let app = RustyX::new();
    
    // Image uploader
    let image_uploader = Uploader::new(
        UploadConfig::new()
            .destination("./uploads/images")
            .images_only()
            .max_file_size_mb(5)
    );
    
    // Document uploader
    let doc_uploader = Uploader::new(
        UploadConfig::new()
            .destination("./uploads/docs")
            .documents_only()
            .max_file_size_mb(20)
    );

    // Upload image
    app.post("/upload/image", move |req, res| {
        let uploader = image_uploader.clone();
        async move {
            // ... upload logic
        }
    });
    
    // Upload document
    app.post("/upload/document", move |req, res| {
        let uploader = doc_uploader.clone();
        async move {
            // ... upload logic
        }
    });

    app.listen(3000).await
}
```

---

## Testing with cURL

### Single File

```bash
curl -X POST http://localhost:3000/upload \
  -F "file=@/path/to/image.png"
```

### Multiple Files

```bash
curl -X POST http://localhost:3000/upload-multiple \
  -F "files=@image1.png" \
  -F "files=@image2.jpg" \
  -F "files=@document.pdf"
```

---

## Helper Functions

### Get MIME Type

```rust
use rustyx::upload::get_mime_type;

let mime = get_mime_type("png");  // "image/png"
let mime = get_mime_type("pdf");  // "application/pdf"
```

### Check File Type

```rust
use rustyx::upload::{is_image, is_document, is_video, is_audio};

is_image("image/png");      // true
is_document("application/pdf");  // true
is_video("video/mp4");      // true
is_audio("audio/mp3");      // true
```

### Parse Boundary

```rust
let content_type = "multipart/form-data; boundary=----WebKitFormBoundary";
let boundary = parse_boundary(content_type);
// Some("----WebKitFormBoundary")
```

---

## Best Practices

1. **Always validate file types** - Use `allowed_types` or `allowed_extensions`
2. **Set size limits** - Prevent large file uploads
3. **Use UUID naming** - Prevents filename conflicts
4. **Create separate uploaders** - Different configs for images vs documents
5. **Handle errors gracefully** - Return meaningful error messages

---

## Next Steps

- [Middleware Guide](./MIDDLEWARE.md)
- [API Reference](./API.md)
- [Deployment Guide](./DEPLOYMENT.md)

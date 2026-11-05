//! Static File Serving
//!
//! Middleware for serving static files.

use crate::request::Request;
use crate::response::Response;
use std::path::{Path, PathBuf};
use tokio::fs;

/// Static file server configuration
#[derive(Debug, Clone)]
pub struct StaticConfig {
    /// Root directory for static files
    pub root: PathBuf,
    /// Index file name
    pub index: String,
    /// Enable directory listing
    pub directory_listing: bool,
    /// Cache control max-age in seconds
    pub max_age: u32,
    /// Enable gzip compression
    pub gzip: bool,
}

impl Default for StaticConfig {
    fn default() -> Self {
        Self {
            root: PathBuf::from("public"),
            index: "index.html".to_string(),
            directory_listing: false,
            max_age: 3600,
            gzip: true,
        }
    }
}

impl StaticConfig {
    /// Create a new static config with root directory
    pub fn new(root: &str) -> Self {
        Self {
            root: PathBuf::from(root),
            ..Default::default()
        }
    }

    /// Set index file
    pub fn index(mut self, index: &str) -> Self {
        self.index = index.to_string();
        self
    }

    /// Enable directory listing
    pub fn directory_listing(mut self, enabled: bool) -> Self {
        self.directory_listing = enabled;
        self
    }

    /// Set cache max-age
    pub fn max_age(mut self, seconds: u32) -> Self {
        self.max_age = seconds;
        self
    }
}

/// Serve a static file
pub async fn serve_file(path: &Path) -> Option<(Vec<u8>, String)> {
    if let Ok(content) = fs::read(path).await {
        let mime_type = get_mime_type(path);
        Some((content, mime_type))
    } else {
        None
    }
}

/// Get MIME type from file extension
pub fn get_mime_type(path: &Path) -> String {
    let extension = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");

    match extension.to_lowercase().as_str() {
        // Text
        "html" | "htm" => "text/html; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "js" | "mjs" => "application/javascript; charset=utf-8",
        "json" => "application/json; charset=utf-8",
        "xml" => "application/xml; charset=utf-8",
        "txt" => "text/plain; charset=utf-8",
        "md" => "text/markdown; charset=utf-8",
        "csv" => "text/csv; charset=utf-8",
        
        // Images
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "svg" => "image/svg+xml",
        "ico" => "image/x-icon",
        "webp" => "image/webp",
        "avif" => "image/avif",
        
        // Fonts
        "woff" => "font/woff",
        "woff2" => "font/woff2",
        "ttf" => "font/ttf",
        "otf" => "font/otf",
        "eot" => "application/vnd.ms-fontobject",
        
        // Documents
        "pdf" => "application/pdf",
        "doc" => "application/msword",
        "docx" => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        "xls" => "application/vnd.ms-excel",
        "xlsx" => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        
        // Archives
        "zip" => "application/zip",
        "tar" => "application/x-tar",
        "gz" => "application/gzip",
        "rar" => "application/vnd.rar",
        
        // Media
        "mp3" => "audio/mpeg",
        "wav" => "audio/wav",
        "ogg" => "audio/ogg",
        "mp4" => "video/mp4",
        "webm" => "video/webm",
        "avi" => "video/x-msvideo",
        
        // WebAssembly
        "wasm" => "application/wasm",
        
        // Default
        _ => "application/octet-stream",
    }
    .to_string()
}

/// Create static file serving handler
///
/// # Example
///
/// ```rust,ignore
/// use nitrox::static_files::{static_handler, StaticConfig};
///
/// let config = StaticConfig::new("./public");
/// app.get("/static/*", static_handler(config));
/// ```
pub fn static_handler<S: Send + Sync + 'static>(
    config: StaticConfig,
) -> impl Fn(Request<S>, Response) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> + Send + Sync + Clone + 'static {
    move |req: Request<S>, res: Response| {
        let config = config.clone();
        
        Box::pin(async move {
            let request_path = req.path();
            
            // Remove leading /static/ or similar prefix
            let file_path = request_path
                .trim_start_matches('/')
                .trim_start_matches("static/");
            
            let canonical_root = match config.root.canonicalize() {
                Ok(r) => r,
                Err(_) => {
                    return res.status(404).json(serde_json::json!({
                        "error": "Not Found",
                        "message": "Static root directory not found"
                    }));
                }
            };

            let target_path = config.root.join(file_path);
            let canonical_target = match target_path.canonicalize() {
                Ok(t) => t,
                Err(_) => {
                    return res.status(404).json(serde_json::json!({
                        "error": "Not Found",
                        "message": "File not found"
                    }));
                }
            };

            // Security check - prevent path traversal
            if !canonical_target.starts_with(&canonical_root) {
                return res.status(403).json(serde_json::json!({
                    "error": "Forbidden",
                    "message": "Access denied"
                }));
            }

            // Check if path is a directory
            if canonical_target.is_dir() {
                let index_path = canonical_target.join(&config.index);
                if let Ok(canonical_index) = index_path.canonicalize() {
                    if canonical_index.starts_with(&canonical_root) {
                        if let Some((content, mime)) = serve_file(&canonical_index).await {
                            return res
                                .status(200)
                                .header("Content-Type", &mime)
                                .header("Cache-Control", &format!("max-age={}", config.max_age))
                                .send_bytes(content);
                        }
                    }
                }
                
                return res.status(404).json(serde_json::json!({
                    "error": "Not Found",
                    "message": "File not found"
                }));
            }

            // Serve the file
            if let Some((content, mime)) = serve_file(&canonical_target).await {
                res.status(200)
                    .header("Content-Type", &mime)
                    .header("Cache-Control", &format!("max-age={}", config.max_age))
                    .send_bytes(content)
            } else {
                res.status(404).json(serde_json::json!({
                    "error": "Not Found",
                    "message": "File not found"
                }))
            }
        })
    }
}

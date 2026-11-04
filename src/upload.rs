//! File Upload Module (Multer-like)
//!
//! Provides file upload functionality similar to Express's Multer.
//!
//! # Features
//!
//! - Single and multiple file uploads
//! - File type validation
//! - File size limits
//! - Disk and memory storage
//! - Custom file naming
//!
//! # Example
//!
//! ```rust,ignore
//! use rustyx::upload::{Uploader, UploadConfig, StorageType};
//!
//! let uploader = Uploader::new(
//!     UploadConfig::new()
//!         .destination("./uploads")
//!         .max_file_size(5 * 1024 * 1024) // 5MB
//!         .allowed_types(vec!["image/png", "image/jpeg", "application/pdf"])
//! );
//!
//! app.post("/upload", move |req, res| {
//!     let uploader = uploader.clone();
//!     async move {
//!         match uploader.single(&req, "file").await {
//!             Ok(file) => res.json(json!({
//!                 "filename": file.filename,
//!                 "size": file.size
//!             })),
//!             Err(e) => res.bad_request(&e.to_string())
//!         }
//!     }
//! });
//! ```

use std::path::{Path, PathBuf};
use tokio::fs;
use tokio::io::AsyncWriteExt;
use uuid::Uuid;

/// Uploaded file information
#[derive(Debug, Clone)]
pub struct UploadedFile {
    /// Original filename from the client
    pub original_name: String,
    /// Saved filename on disk (may be different from original)
    pub filename: String,
    /// Full path to the saved file
    pub path: PathBuf,
    /// MIME type of the file
    pub mimetype: String,
    /// File size in bytes
    pub size: usize,
    /// Field name from the form
    pub field_name: String,
    /// File extension
    pub extension: String,
}

/// Storage type for uploaded files
#[derive(Debug, Clone)]
pub enum StorageType {
    /// Store files on disk
    Disk {
        /// Destination directory
        destination: PathBuf,
    },
    /// Keep files in memory
    Memory,
}

impl Default for StorageType {
    fn default() -> Self {
        StorageType::Disk {
            destination: PathBuf::from("./uploads"),
        }
    }
}

/// File naming strategy
#[derive(Debug, Clone)]
pub enum FileNaming {
    /// Keep original filename
    Original,
    /// Use UUID for filename
    Uuid,
    /// Use UUID with original extension
    UuidWithExtension,
    /// Use timestamp with original extension
    TimestampWithExtension,
    /// Custom prefix with UUID
    CustomPrefix(String),
}

impl Default for FileNaming {
    fn default() -> Self {
        FileNaming::UuidWithExtension
    }
}

/// Upload configuration
#[derive(Debug, Clone)]
pub struct UploadConfig {
    /// Storage type (disk or memory)
    pub storage: StorageType,
    /// Maximum file size in bytes (default: 10MB)
    pub max_file_size: usize,
    /// Maximum number of files (for multiple uploads)
    pub max_files: usize,
    /// Allowed MIME types (empty = allow all)
    pub allowed_types: Vec<String>,
    /// Allowed file extensions (empty = allow all)
    pub allowed_extensions: Vec<String>,
    /// File naming strategy
    pub naming: FileNaming,
    /// Create destination directory if it doesn't exist
    pub create_dir: bool,
    /// Preserve file extension
    pub preserve_extension: bool,
}

impl Default for UploadConfig {
    fn default() -> Self {
        Self {
            storage: StorageType::default(),
            max_file_size: 10 * 1024 * 1024, // 10MB
            max_files: 10,
            allowed_types: Vec::new(),
            allowed_extensions: Vec::new(),
            naming: FileNaming::default(),
            create_dir: true,
            preserve_extension: true,
        }
    }
}

impl UploadConfig {
    /// Create a new upload configuration
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the destination directory
    pub fn destination(mut self, path: &str) -> Self {
        self.storage = StorageType::Disk {
            destination: PathBuf::from(path),
        };
        self
    }

    /// Use memory storage instead of disk
    pub fn memory(mut self) -> Self {
        self.storage = StorageType::Memory;
        self
    }

    /// Set maximum file size in bytes
    pub fn max_file_size(mut self, size: usize) -> Self {
        self.max_file_size = size;
        self
    }

    /// Set maximum file size in MB
    pub fn max_file_size_mb(mut self, mb: usize) -> Self {
        self.max_file_size = mb * 1024 * 1024;
        self
    }

    /// Set maximum number of files for multiple uploads
    pub fn max_files(mut self, count: usize) -> Self {
        self.max_files = count;
        self
    }

    /// Set allowed MIME types
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let config = UploadConfig::new()
    ///     .allowed_types(vec!["image/png", "image/jpeg", "application/pdf"]);
    /// ```
    pub fn allowed_types(mut self, types: Vec<&str>) -> Self {
        self.allowed_types = types.iter().map(|s| s.to_string()).collect();
        self
    }

    /// Set allowed file extensions
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let config = UploadConfig::new()
    ///     .allowed_extensions(vec!["png", "jpg", "jpeg", "pdf"]);
    /// ```
    pub fn allowed_extensions(mut self, extensions: Vec<&str>) -> Self {
        self.allowed_extensions = extensions.iter().map(|s| s.to_lowercase()).collect();
        self
    }

    /// Allow only image files
    pub fn images_only(mut self) -> Self {
        self.allowed_types = vec![
            "image/png".to_string(),
            "image/jpeg".to_string(),
            "image/jpg".to_string(),
            "image/gif".to_string(),
            "image/webp".to_string(),
            "image/svg+xml".to_string(),
        ];
        self.allowed_extensions = vec![
            "png".to_string(),
            "jpg".to_string(),
            "jpeg".to_string(),
            "gif".to_string(),
            "webp".to_string(),
            "svg".to_string(),
        ];
        self
    }

    /// Allow only document files
    pub fn documents_only(mut self) -> Self {
        self.allowed_types = vec![
            "application/pdf".to_string(),
            "application/msword".to_string(),
            "application/vnd.openxmlformats-officedocument.wordprocessingml.document".to_string(),
            "application/vnd.ms-excel".to_string(),
            "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet".to_string(),
            "text/plain".to_string(),
        ];
        self.allowed_extensions = vec![
            "pdf".to_string(),
            "doc".to_string(),
            "docx".to_string(),
            "xls".to_string(),
            "xlsx".to_string(),
            "txt".to_string(),
        ];
        self
    }

    /// Set file naming strategy
    pub fn naming(mut self, naming: FileNaming) -> Self {
        self.naming = naming;
        self
    }

    /// Use original filenames
    pub fn keep_original_name(mut self) -> Self {
        self.naming = FileNaming::Original;
        self
    }

    /// Use UUID filenames
    pub fn use_uuid(mut self) -> Self {
        self.naming = FileNaming::UuidWithExtension;
        self
    }
}

/// Upload error types
#[derive(Debug, Clone)]
pub enum UploadError {
    /// File too large
    FileTooLarge { max: usize, actual: usize },
    /// File type not allowed
    TypeNotAllowed { mimetype: String },
    /// Extension not allowed
    ExtensionNotAllowed { extension: String },
    /// Too many files
    TooManyFiles { max: usize, actual: usize },
    /// No file provided
    NoFile,
    /// Field not found
    FieldNotFound { field: String },
    /// IO error
    IoError(String),
    /// Parse error
    ParseError(String),
}

impl std::fmt::Display for UploadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UploadError::FileTooLarge { max, actual } => {
                write!(f, "File too large: {} bytes (max: {} bytes)", actual, max)
            }
            UploadError::TypeNotAllowed { mimetype } => {
                write!(f, "File type not allowed: {}", mimetype)
            }
            UploadError::ExtensionNotAllowed { extension } => {
                write!(f, "File extension not allowed: {}", extension)
            }
            UploadError::TooManyFiles { max, actual } => {
                write!(f, "Too many files: {} (max: {})", actual, max)
            }
            UploadError::NoFile => write!(f, "No file provided"),
            UploadError::FieldNotFound { field } => {
                write!(f, "Field not found: {}", field)
            }
            UploadError::IoError(msg) => write!(f, "IO error: {}", msg),
            UploadError::ParseError(msg) => write!(f, "Parse error: {}", msg),
        }
    }
}

impl std::error::Error for UploadError {}

/// File uploader (similar to Multer)
#[derive(Debug, Clone)]
pub struct Uploader {
    config: UploadConfig,
}

impl Uploader {
    /// Create a new uploader with configuration
    pub fn new(config: UploadConfig) -> Self {
        Self { config }
    }

    /// Create a simple uploader for a destination directory
    pub fn disk(destination: &str) -> Self {
        Self::new(UploadConfig::new().destination(destination))
    }

    /// Create a memory uploader
    pub fn memory() -> Self {
        Self::new(UploadConfig::new().memory())
    }

    /// Get the configuration
    pub fn config(&self) -> &UploadConfig {
        &self.config
    }

    /// Generate filename based on naming strategy
    fn generate_filename(&self, original: &str) -> String {
        let extension = Path::new(original)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");

        match &self.config.naming {
            FileNaming::Original => original.to_string(),
            FileNaming::Uuid => Uuid::new_v4().to_string(),
            FileNaming::UuidWithExtension => {
                if extension.is_empty() {
                    Uuid::new_v4().to_string()
                } else {
                    format!("{}.{}", Uuid::new_v4(), extension)
                }
            }
            FileNaming::TimestampWithExtension => {
                let timestamp = chrono::Utc::now().timestamp_millis();
                if extension.is_empty() {
                    timestamp.to_string()
                } else {
                    format!("{}.{}", timestamp, extension)
                }
            }
            FileNaming::CustomPrefix(prefix) => {
                if extension.is_empty() {
                    format!("{}_{}", prefix, Uuid::new_v4())
                } else {
                    format!("{}_{}.{}", prefix, Uuid::new_v4(), extension)
                }
            }
        }
    }

    /// Validate file
    fn validate(&self, mimetype: &str, extension: &str, size: usize) -> Result<(), UploadError> {
        // Check file size
        if size > self.config.max_file_size {
            return Err(UploadError::FileTooLarge {
                max: self.config.max_file_size,
                actual: size,
            });
        }

        // Check MIME type
        if !self.config.allowed_types.is_empty()
            && !self.config.allowed_types.iter().any(|t| t == mimetype)
        {
            return Err(UploadError::TypeNotAllowed {
                mimetype: mimetype.to_string(),
            });
        }

        // Check extension
        if !self.config.allowed_extensions.is_empty()
            && !self
                .config
                .allowed_extensions
                .iter()
                .any(|e| e == &extension.to_lowercase())
        {
            return Err(UploadError::ExtensionNotAllowed {
                extension: extension.to_string(),
            });
        }

        Ok(())
    }

    /// Save file to disk
    async fn save_to_disk(
        &self,
        destination: &Path,
        filename: &str,
        data: &[u8],
    ) -> Result<PathBuf, UploadError> {
        // Create directory if needed
        if self.config.create_dir && !destination.exists() {
            fs::create_dir_all(destination)
                .await
                .map_err(|e| UploadError::IoError(e.to_string()))?;
        }

        let file_path = destination.join(filename);

        // Write file
        let mut file = fs::File::create(&file_path)
            .await
            .map_err(|e| UploadError::IoError(e.to_string()))?;

        file.write_all(data)
            .await
            .map_err(|e| UploadError::IoError(e.to_string()))?;

        Ok(file_path)
    }

    /// Upload a single file from multipart form data
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let file = uploader.upload_single("avatar", data, "image.png", "image/png").await?;
    /// ```
    pub async fn upload_single(
        &self,
        field_name: &str,
        data: Vec<u8>,
        original_name: &str,
        mimetype: &str,
    ) -> Result<UploadedFile, UploadError> {
        let extension = Path::new(original_name)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_string();

        // Validate
        self.validate(mimetype, &extension, data.len())?;

        // Generate filename
        let filename = self.generate_filename(original_name);

        // Save file
        let path = match &self.config.storage {
            StorageType::Disk { destination } => {
                self.save_to_disk(destination, &filename, &data).await?
            }
            StorageType::Memory => PathBuf::new(), // No path for memory storage
        };

        Ok(UploadedFile {
            original_name: original_name.to_string(),
            filename,
            path,
            mimetype: mimetype.to_string(),
            size: data.len(),
            field_name: field_name.to_string(),
            extension,
        })
    }

    /// Upload multiple files
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let files = uploader.upload_multiple(files_data).await?;
    /// ```
    pub async fn upload_multiple(
        &self,
        files: Vec<(String, Vec<u8>, String, String)>, // (field, data, name, mimetype)
    ) -> Result<Vec<UploadedFile>, UploadError> {
        // Check file count
        if files.len() > self.config.max_files {
            return Err(UploadError::TooManyFiles {
                max: self.config.max_files,
                actual: files.len(),
            });
        }

        let mut uploaded = Vec::new();

        for (field_name, data, original_name, mimetype) in files {
            let file = self
                .upload_single(&field_name, data, &original_name, &mimetype)
                .await?;
            uploaded.push(file);
        }

        Ok(uploaded)
    }
}

impl Default for Uploader {
    fn default() -> Self {
        Self::new(UploadConfig::default())
    }
}

/// Parse multipart form data boundary from content-type header
pub fn parse_boundary(content_type: &str) -> Option<String> {
    content_type
        .split(';')
        .find(|part| part.trim().starts_with("boundary="))
        .map(|part| {
            part.trim()
                .trim_start_matches("boundary=")
                .trim_matches('"')
                .to_string()
        })
}

/// Simple multipart form data parser
#[derive(Debug, Clone)]
pub struct MultipartField {
    /// Field name
    pub name: String,
    /// Filename (for file fields)
    pub filename: Option<String>,
    /// Content type
    pub content_type: Option<String>,
    /// Field data
    pub data: Vec<u8>,
}

/// Parse multipart form data
///
/// Note: This is a simplified parser. For production use,
/// consider using a dedicated multipart parsing library.
pub fn parse_multipart(body: &[u8], boundary: &str) -> Result<Vec<MultipartField>, UploadError> {
    let mut fields = Vec::new();
    let boundary_bytes = format!("--{}", boundary);
    let end_boundary = format!("--{}--", boundary);

    // Convert to string for easier parsing
    // Note: This may fail for binary data in some edge cases
    let body_str = String::from_utf8_lossy(body);

    for part in body_str.split(&boundary_bytes) {
        let part = part.trim();
        if part.is_empty() || part == "--" || part.starts_with(&end_boundary) {
            continue;
        }

        // Split headers and content
        if let Some(header_end) = part.find("\r\n\r\n") {
            let headers = &part[..header_end];
            let content = &part[header_end + 4..];

            // Remove trailing CRLF
            let content = content.trim_end_matches("\r\n");

            // Parse Content-Disposition header
            let mut name = String::new();
            let mut filename = None;
            let mut content_type = None;

            for line in headers.lines() {
                if line.to_lowercase().starts_with("content-disposition:") {
                    // Parse name
                    if let Some(name_start) = line.find("name=\"") {
                        let rest = &line[name_start + 6..];
                        if let Some(name_end) = rest.find('"') {
                            name = rest[..name_end].to_string();
                        }
                    }
                    // Parse filename
                    if let Some(fname_start) = line.find("filename=\"") {
                        let rest = &line[fname_start + 10..];
                        if let Some(fname_end) = rest.find('"') {
                            filename = Some(rest[..fname_end].to_string());
                        }
                    }
                } else if line.to_lowercase().starts_with("content-type:") {
                    content_type = Some(line[13..].trim().to_string());
                }
            }

            if !name.is_empty() {
                fields.push(MultipartField {
                    name,
                    filename,
                    content_type,
                    data: content.as_bytes().to_vec(),
                });
            }
        }
    }

    Ok(fields)
}

/// Get MIME type from file extension
pub fn get_mime_type(extension: &str) -> &'static str {
    match extension.to_lowercase().as_str() {
        // Images
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "svg" => "image/svg+xml",
        "ico" => "image/x-icon",
        "bmp" => "image/bmp",
        "tiff" | "tif" => "image/tiff",

        // Documents
        "pdf" => "application/pdf",
        "doc" => "application/msword",
        "docx" => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        "xls" => "application/vnd.ms-excel",
        "xlsx" => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        "ppt" => "application/vnd.ms-powerpoint",
        "pptx" => "application/vnd.openxmlformats-officedocument.presentationml.presentation",
        "txt" => "text/plain",
        "csv" => "text/csv",
        "json" => "application/json",
        "xml" => "application/xml",

        // Audio
        "mp3" => "audio/mpeg",
        "wav" => "audio/wav",
        "ogg" => "audio/ogg",
        "m4a" => "audio/mp4",

        // Video
        "mp4" => "video/mp4",
        "webm" => "video/webm",
        "avi" => "video/x-msvideo",
        "mov" => "video/quicktime",
        "mkv" => "video/x-matroska",

        // Archives
        "zip" => "application/zip",
        "rar" => "application/vnd.rar",
        "7z" => "application/x-7z-compressed",
        "tar" => "application/x-tar",
        "gz" => "application/gzip",

        // Default
        _ => "application/octet-stream",
    }
}

/// Check if MIME type is an image
pub fn is_image(mimetype: &str) -> bool {
    mimetype.starts_with("image/")
}

/// Check if MIME type is a document
pub fn is_document(mimetype: &str) -> bool {
    matches!(
        mimetype,
        "application/pdf"
            | "application/msword"
            | "application/vnd.openxmlformats-officedocument.wordprocessingml.document"
            | "application/vnd.ms-excel"
            | "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"
            | "text/plain"
    )
}

/// Check if MIME type is a video
pub fn is_video(mimetype: &str) -> bool {
    mimetype.starts_with("video/")
}

/// Check if MIME type is audio
pub fn is_audio(mimetype: &str) -> bool {
    mimetype.starts_with("audio/")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_upload_config() {
        let config = UploadConfig::new()
            .destination("./uploads")
            .max_file_size_mb(5)
            .allowed_extensions(vec!["png", "jpg"]);

        assert_eq!(config.max_file_size, 5 * 1024 * 1024);
        assert_eq!(config.allowed_extensions.len(), 2);
    }

    #[test]
    fn test_uploader_validation() {
        let uploader = Uploader::new(
            UploadConfig::new()
                .max_file_size(1024)
                .allowed_extensions(vec!["png"]),
        );

        // Should pass
        assert!(uploader.validate("image/png", "png", 500).is_ok());

        // Should fail - too large
        assert!(uploader.validate("image/png", "png", 2000).is_err());

        // Should fail - wrong extension
        assert!(uploader.validate("image/jpeg", "jpg", 500).is_err());
    }

    #[test]
    fn test_get_mime_type() {
        assert_eq!(get_mime_type("png"), "image/png");
        assert_eq!(get_mime_type("pdf"), "application/pdf");
        assert_eq!(get_mime_type("unknown"), "application/octet-stream");
    }
}

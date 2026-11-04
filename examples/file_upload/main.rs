//! File Upload Example
//!
//! This example demonstrates how to handle file uploads using RustyX.
//!
//! ## Testing with curl
//!
//! Single file upload:
//! ```bash
//! curl -X POST http://localhost:3000/upload \
//!   -F "file=@/path/to/image.png"
//! ```
//!
//! Multiple file upload:
//! ```bash
//! curl -X POST http://localhost:3000/upload-multiple \
//!   -F "files=@/path/to/image1.png" \
//!   -F "files=@/path/to/image2.jpg"
//! ```

use rustyx::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt().with_env_filter("info").init();

    let app = RustyX::new();

    // Add middleware
    app.use_middleware(logger());
    app.use_middleware(cors("*"));

    // Root route with upload form
    app.get("/", |_req, res| async move {
        res.html(
            r#"
<!DOCTYPE html>
<html>
<head>
    <title>RustyX File Upload</title>
    <style>
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            max-width: 600px;
            margin: 50px auto;
            padding: 20px;
            background: #1a1a2e;
            color: #eee;
        }
        h1 { color: #ff6b35; }
        .upload-form {
            background: #16213e;
            padding: 30px;
            border-radius: 10px;
            margin: 20px 0;
        }
        input[type="file"] {
            display: block;
            margin: 15px 0;
            padding: 10px;
            background: #0f3460;
            border: none;
            border-radius: 5px;
            color: #fff;
            width: 100%;
        }
        button {
            background: #ff6b35;
            color: white;
            padding: 12px 24px;
            border: none;
            border-radius: 5px;
            cursor: pointer;
            font-size: 16px;
        }
        button:hover { background: #e55a2b; }
        #result {
            margin-top: 20px;
            padding: 15px;
            background: #0f3460;
            border-radius: 5px;
            display: none;
        }
        pre { white-space: pre-wrap; word-wrap: break-word; }
    </style>
</head>
<body>
    <h1>🚀 RustyX File Upload</h1>
    
    <div class="upload-form">
        <h3>Single File Upload</h3>
        <form id="singleForm">
            <input type="file" name="file" id="singleFile" accept="image/*,.pdf">
            <button type="submit">Upload File</button>
        </form>
    </div>

    <div class="upload-form">
        <h3>Multiple File Upload</h3>
        <form id="multipleForm">
            <input type="file" name="files" id="multipleFiles" multiple accept="image/*,.pdf">
            <button type="submit">Upload Files</button>
        </form>
    </div>

    <div id="result">
        <h4>Response:</h4>
        <pre id="responseText"></pre>
    </div>

    <script>
        document.getElementById('singleForm').addEventListener('submit', async (e) => {
            e.preventDefault();
            const formData = new FormData();
            formData.append('file', document.getElementById('singleFile').files[0]);
            
            const response = await fetch('/upload', {
                method: 'POST',
                body: formData
            });
            const data = await response.json();
            showResult(data);
        });

        document.getElementById('multipleForm').addEventListener('submit', async (e) => {
            e.preventDefault();
            const formData = new FormData();
            const files = document.getElementById('multipleFiles').files;
            for (let i = 0; i < files.length; i++) {
                formData.append('files', files[i]);
            }
            
            const response = await fetch('/upload-multiple', {
                method: 'POST',
                body: formData
            });
            const data = await response.json();
            showResult(data);
        });

        function showResult(data) {
            document.getElementById('result').style.display = 'block';
            document.getElementById('responseText').textContent = JSON.stringify(data, null, 2);
        }
    </script>
</body>
</html>
        "#,
        )
    });

    // Single file upload endpoint
    let single_uploader = Uploader::new(
        UploadConfig::new()
            .destination("./uploads")
            .max_file_size_mb(10)
            .allowed_extensions(vec!["png", "jpg", "jpeg", "gif", "webp", "pdf"]),
    );

    app.post("/upload", move |req, res| {
        let uploader = single_uploader.clone();
        async move {
            // Get content type header
            let content_type = req.content_type().unwrap_or_default().to_string();

            // Check if it's multipart form data
            if !content_type.contains("multipart/form-data") {
                return res.bad_request("Content-Type must be multipart/form-data");
            }

            // Parse boundary
            let boundary = match parse_boundary(&content_type) {
                Some(b) => b,
                None => return res.bad_request("Missing boundary in Content-Type"),
            };

            // Parse multipart form data
            let fields = match parse_multipart(req.body(), &boundary) {
                Ok(f) => f,
                Err(e) => return res.bad_request(&format!("Failed to parse form data: {}", e)),
            };

            // Find file field
            for field in fields {
                if field.name == "file" {
                    if let Some(filename) = field.filename {
                        let mimetype = field
                            .content_type
                            .unwrap_or_else(|| "application/octet-stream".to_string());

                        match uploader
                            .upload_single(&field.name, field.data, &filename, &mimetype)
                            .await
                        {
                            Ok(file) => {
                                info!("File uploaded: {} ({} bytes)", file.filename, file.size);
                                return res.json(json!({
                                    "success": true,
                                    "message": "File uploaded successfully",
                                    "file": {
                                        "filename": file.filename,
                                        "originalName": file.original_name,
                                        "size": file.size,
                                        "mimetype": file.mimetype,
                                        "extension": file.extension,
                                        "path": file.path.to_string_lossy()
                                    }
                                }));
                            }
                            Err(e) => {
                                error!("Upload failed: {}", e);
                                return res.status(400).json(json!({
                                    "success": false,
                                    "error": e.to_string()
                                }));
                            }
                        }
                    }
                }
            }

            res.bad_request("No file provided in 'file' field")
        }
    });

    // Multiple file upload endpoint
    let multi_uploader = Uploader::new(
        UploadConfig::new()
            .destination("./uploads")
            .max_file_size_mb(10)
            .max_files(5)
            .images_only(),
    );

    app.post("/upload-multiple", move |req, res| {
        let uploader = multi_uploader.clone();
        async move {
            let content_type = req.content_type().unwrap_or_default().to_string();

            if !content_type.contains("multipart/form-data") {
                return res.bad_request("Content-Type must be multipart/form-data");
            }

            let boundary = match parse_boundary(&content_type) {
                Some(b) => b,
                None => return res.bad_request("Missing boundary"),
            };

            let fields = match parse_multipart(req.body(), &boundary) {
                Ok(f) => f,
                Err(e) => return res.bad_request(&format!("Parse error: {}", e)),
            };

            // Collect all file fields
            let mut files_data = Vec::new();
            for field in fields {
                if let Some(filename) = field.filename {
                    let mimetype = field
                        .content_type
                        .unwrap_or_else(|| "application/octet-stream".to_string());
                    files_data.push((field.name, field.data, filename, mimetype));
                }
            }

            if files_data.is_empty() {
                return res.bad_request("No files provided");
            }

            match uploader.upload_multiple(files_data).await {
                Ok(files) => {
                    info!("Uploaded {} files", files.len());
                    res.json(json!({
                        "success": true,
                        "message": format!("{} files uploaded successfully", files.len()),
                        "files": files.iter().map(|f| json!({
                            "filename": f.filename,
                            "originalName": f.original_name,
                            "size": f.size,
                            "mimetype": f.mimetype
                        })).collect::<Vec<_>>()
                    }))
                }
                Err(e) => {
                    error!("Multi-upload failed: {}", e);
                    res.status(400).json(json!({
                        "success": false,
                        "error": e.to_string()
                    }))
                }
            }
        }
    });

    // List uploaded files
    app.get("/files", |_req, res| async move {
        match tokio::fs::read_dir("./uploads").await {
            Ok(mut entries) => {
                let mut files = Vec::new();
                while let Ok(Some(entry)) = entries.next_entry().await {
                    if let Ok(metadata) = entry.metadata().await {
                        files.push(json!({
                            "name": entry.file_name().to_string_lossy(),
                            "size": metadata.len(),
                            "isFile": metadata.is_file()
                        }));
                    }
                }
                res.json(json!({
                    "files": files,
                    "count": files.len()
                }))
            }
            Err(_) => res.json(json!({
                "files": [],
                "count": 0,
                "message": "Upload directory not found or empty"
            })),
        }
    });

    info!("🚀 File Upload Server running at http://localhost:3000");
    info!("   Open browser to test file uploads");

    app.listen(3000).await
}

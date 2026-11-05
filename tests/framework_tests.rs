use nitrox::prelude::*;
use nitrox::upload::parse_multipart;

#[tokio::test]
async fn test_param_matching_direct() {
    let mut router = Router::new();
    router.get("/users/:id", |req, res| async move {
        let id = req.param("id").cloned().unwrap_or_default();
        res.json(json!({ "user_id": id }))
    });

    let (handler, params) = router
        .find_route(&hyper::Method::GET, "/users/42")
        .expect("Route with :id must match");

    assert_eq!(params.get("id"), Some(&"42".to_string()));
    let _ = handler;
}

#[tokio::test]
async fn test_multipart_binary_safe() {
    let boundary = "---------------------------974767299852498929531610575";
    let binary_bytes: Vec<u8> = vec![
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0xFF, 0xFE,
    ];

    let mut body = Vec::new();
    body.extend_from_slice(format!("--{}\r\n", boundary).as_bytes());
    body.extend_from_slice(
        b"Content-Disposition: form-data; name=\"file\"; filename=\"test.png\"\r\n",
    );
    body.extend_from_slice(b"Content-Type: image/png\r\n\r\n");
    body.extend_from_slice(&binary_bytes);
    body.extend_from_slice(format!("\r\n--{}--\r\n", boundary).as_bytes());

    let fields = parse_multipart(&body, boundary).expect("Should parse multipart");
    assert_eq!(fields.len(), 1);
    assert_eq!(fields[0].name, "file");
    assert_eq!(fields[0].filename, Some("test.png".to_string()));
    assert_eq!(fields[0].data, binary_bytes);
}

#[tokio::test]
async fn test_upload_filename_traversal_sanitized() {
    let uploader = Uploader::new(
        UploadConfig::new()
            .keep_original_name()
            .destination("./test_uploads"),
    );

    let malicious_filename = "../../../etc/cron.d/malicious";
    let file = uploader
        .upload_single("file", vec![1, 2, 3], malicious_filename, "text/plain")
        .await
        .expect("Upload should sanitize filename");

    assert_eq!(file.filename, "malicious");
    assert!(!file.filename.contains(".."));
    assert!(!file.filename.contains('/'));
    assert!(!file.filename.contains('\\'));

    // Clean up
    let _ = tokio::fs::remove_dir_all("./test_uploads").await;
}


#[tokio::test]
async fn test_state_direct_access_and_router() {
    #[derive(Clone)]
    struct MockRedis {
        connected: bool,
    }

    struct TestAppState {
        redis: MockRedis,
        env: String,
    }

    let mut router = Router::<TestAppState>::with_state();
    router.get("/info", |req, res| async move {
        let redis_status = req.state.redis.connected;
        let env = &req.state.env;
        res.json(json!({
            "redis": redis_status,
            "env": env
        }))
    });

    let (handler, _params) = router
        .find_route(&hyper::Method::GET, "/info")
        .expect("Route /info should match");

    let _ = handler;
}

#[tokio::test]
async fn test_app_with_state_instantiation() {
    struct CustomState {
        counter: std::sync::atomic::AtomicUsize,
        service_name: String,
    }

    let state = CustomState {
        counter: std::sync::atomic::AtomicUsize::new(42),
        service_name: "payment-gateway".to_string(),
    };

    let app = Nitrox::with_state(state);
    assert_eq!(app.state().service_name, "payment-gateway");
    assert_eq!(app.state().counter.load(std::sync::atomic::Ordering::SeqCst), 42);

    app.get("/status", |req, res| async move {
        let count = req.state.counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        res.json(json!({ "count": count, "service": req.state.service_name }))
    });
}

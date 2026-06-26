use axum::{
    extract::Path,
    routing::get,
    Router,
    response::Html,
};
use std::net::SocketAddr;

async fn handle_root() -> Html<String> {
    handle_request("index.php".to_string()).await
}

async fn handle_path(Path(path): Path<String>) -> Html<String> {
    handle_request(path).await
}

async fn handle_request(path: String) -> Html<String> {
    // Sicurezza di base per non uscire dalla cartella public
    if path.contains("..") {
        return Html("<h1>403 Forbidden</h1>".to_string());
    }

    let file_path = if path.is_empty() || path == "/" {
        "index.php".to_string()
    } else {
        path.trim_start_matches('/').to_string()
    };

    let full_path = format!("public/{}", file_path);

    let source = match tokio::fs::read(&full_path).await {
        Ok(s) => s,
        Err(_) => return Html(format!("<h1>404 Not Found: {}</h1>", file_path)),
    };
    
    let result: Result<String, String> = tokio::task::spawn_blocking(move || {
        let registry = php_builtins::registry();
        match php_runtime::run_source_with(file_path.as_bytes(), &source, &registry) {
            Ok(outcome) => {
                Ok(String::from_utf8_lossy(&outcome.rendered).into_owned())
            }
            Err(e) => {
                Err(format!("PHP Parse error in {}: {:?}", file_path, e))
            }
        }
    }).await.unwrap();

    match result {
        Ok(html) => Html(html),
        Err(e) => Html(e),
    }
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(handle_root))
        .route("/*path", get(handle_path));
    
    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    println!("Server in ascolto su http://{}", addr);
    
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

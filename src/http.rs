use axum::{Router, response::Html, routing::get};
use tokio::net::TcpListener;

/// # Errors
/// - if `TcpListener` cannot bind
/// - if axum server cannot be started
pub async fn serve() -> anyhow::Result<()> {
    let router = Router::new().route("/", get(get_root));

    let port = 8502;
    let address = format!("[::]:{port}");

    let listener = TcpListener::bind(address).await?;

    axum::serve(listener, router).await?;

    Ok(())
}

async fn get_root() -> Html<&'static str> {
    Html("<head><title>Willow Application Server</title></head>")
}

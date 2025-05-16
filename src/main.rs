use willow_application_server_rs::{http::serve, trace::init_tracing};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_tracing()?;
    tracing::info!("starting");

    serve().await?;

    Ok(())
}

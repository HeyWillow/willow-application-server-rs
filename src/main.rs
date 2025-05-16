use willow_application_server_rs::{http::serve, trace::init_tracing, willow::worker::WorkerData};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_tracing()?;
    tracing::info!("starting");

    let worker_data = WorkerData::create().await?;
    tracing::debug!("{worker_data:#?}");

    serve().await?;

    Ok(())
}

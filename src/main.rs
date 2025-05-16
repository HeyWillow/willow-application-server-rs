use willow_application_server_rs::{
    http::serve, state::WasState, trace::init_tracing, willow::worker::WorkerData,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_tracing()?;
    tracing::info!("starting");

    let worker_data = WorkerData::create().await?;
    let state = WasState::new(worker_data);

    tracing::debug!("{state:#?}");

    serve().await?;

    Ok(())
}

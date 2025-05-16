use willow_application_server_rs::trace::init_tracing;

fn main() -> anyhow::Result<()> {
    init_tracing()?;
    tracing::info!("starting");

    Ok(())
}

use tracing::level_filters::LevelFilter;
use tracing_subscriber::{EnvFilter, Layer, layer::SubscriberExt, util::SubscriberInitExt};

/// # Errors
/// if the tracing subscriber fails to initialize
pub fn init_tracing() -> anyhow::Result<()> {
    let filter_env = EnvFilter::try_from_env("WAS_LOG")
        .unwrap_or_else(|_| EnvFilter::new("").add_directive(LevelFilter::INFO.into()));

    let layer_fmt = tracing_subscriber::fmt::layer()
        .with_file(true)
        .with_line_number(true)
        .with_target(true)
        .with_thread_names(true)
        .boxed();

    tracing_subscriber::registry()
        .with(filter_env)
        .with(layer_fmt)
        .try_init()?;

    Ok(())
}

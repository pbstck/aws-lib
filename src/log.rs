use tracing_subscriber::EnvFilter;

pub fn setup_log() {
    tracing_subscriber::fmt::fmt()
        .json()
        .flatten_event(true)
        .with_current_span(false)
        .with_span_list(true)
        .with_env_filter(EnvFilter::from_env("LOG_LEVEL"))
        .init();
}

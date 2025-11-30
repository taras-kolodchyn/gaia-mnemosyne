/// Initialize a basic tracing subscriber for logging.
pub fn init_logging() {
    tracing_subscriber::fmt().with_target(false).without_time().init();
}

/// Initialize tracing with an explicit registry and fmt layer.
pub fn init_tracing() {
    use tracing_subscriber::{fmt, prelude::*, registry::Registry};

    let fmt_layer = fmt::layer().without_time();
    Registry::default().with(fmt_layer).init();
}

//! Tracing configuration for debugging and teach-me mode.
//!
//! This module provides a default tracing subscriber configuration that enables
//! all `--teach-me` messages and detailed logging output.
//!
//! # Example
//!
//! ```no_run
//! # #[cfg(feature = "tracing_debug")]
//! cargo_near_build::init_tracing_debug().expect("Failed to initialize tracing");
//! ```

use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

/// Initializes a tracing subscriber with configuration suitable for debugging.
///
/// This function sets up a tracing subscriber that:
/// - Enables all `near_teach_me` target messages (INFO level)
/// - Shows WARN level messages from all sources
/// - Respects RUST_LOG environment variable if set
///
/// This is the same configuration used by `cargo near --teach-me` flag.
///
/// # Errors
///
/// Returns an error if:
/// - The tracing subscriber fails to initialize
/// - The environment filter configuration is invalid
///
/// # Example
///
/// ```no_run
/// # #[cfg(feature = "tracing_debug")]
/// cargo_near_build::init_tracing_debug().expect("Failed to initialize tracing");
/// ```
pub fn init_tracing_debug() -> Result<(), Box<dyn std::error::Error>> {
    use tracing::{Event, Level, Subscriber};
    use tracing_subscriber::{
        fmt::{format::Writer, FmtContext, FormatEvent, FormatFields},
        registry::LookupSpan,
    };

    struct SimpleFormatter;

    impl<S, N> FormatEvent<S, N> for SimpleFormatter
    where
        S: Subscriber + for<'a> LookupSpan<'a>,
        N: for<'a> FormatFields<'a> + 'static,
    {
        fn format_event(
            &self,
            ctx: &FmtContext<'_, S, N>,
            mut writer: Writer<'_>,
            event: &Event<'_>,
        ) -> std::fmt::Result {
            let level = *event.metadata().level();
            let (icon, color_code) = match level {
                Level::TRACE => ("TRACE ", "\x1b[35m"),  // Magenta
                Level::DEBUG => ("DEBUG ", "\x1b[34m"),  // Blue
                Level::INFO => ("", ""),                 // Default
                Level::WARN => ("Warning ", "\x1b[33m"), // Yellow
                Level::ERROR => ("ERROR ", "\x1b[31m"),  // Red
            };

            write!(writer, "{}├  {}", color_code, icon)?;
            write!(writer, "\x1b[0m")?;

            ctx.field_format().format_fields(writer.by_ref(), event)?;

            writeln!(writer)
        }
    }

    let env_filter = EnvFilter::from_default_env()
        .add_directive(Level::WARN.into())
        .add_directive("near_teach_me=info".parse()?)
        .add_directive("near_cli_rs=info".parse()?)
        .add_directive("tracing_instrument=info".parse()?);

    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer().event_format(SimpleFormatter))
        .with(env_filter)
        .init();

    Ok(())
}

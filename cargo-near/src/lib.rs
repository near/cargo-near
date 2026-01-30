#![allow(clippy::large_enum_variant)]

pub use near_cli_rs::{CliResult, GlobalContext, Verbosity};

use strum::{EnumDiscriminants, EnumIter, EnumMessage};

pub mod commands;
pub mod types;

pub(crate) mod posthog_tracking;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = near_cli_rs::GlobalContext)]
pub struct Cmd {
    #[interactive_clap(subcommand)]
    pub opts: Opts,
}

#[derive(Debug, EnumDiscriminants, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = near_cli_rs::GlobalContext)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
#[interactive_clap(disable_back)]
/// Near
pub enum Opts {
    #[strum_discriminants(strum(message = "near"))]
    /// Which subcommand of `near` extension do you want to use?
    Near(NearArgs),
}

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = near_cli_rs::GlobalContext)]
pub struct NearArgs {
    /// TEACH-ME mode, more verbose logging for each action that the CLI performs
    #[interactive_clap(long)]
    teach_me: bool,
    #[interactive_clap(subcommand)]
    pub cmd: self::commands::NearCommand,
}

pub fn setup_tracing(rust_log_env_is_set: bool, teach_me_flag_is_set: bool) -> CliResult {
    use colored::Colorize;
    use tracing::{Event, Level, Subscriber};
    use tracing_indicatif::IndicatifLayer;
    use tracing_indicatif::style::ProgressStyle;
    use tracing_subscriber::EnvFilter;
    use tracing_subscriber::layer::SubscriberExt;
    use tracing_subscriber::util::SubscriberInitExt;
    use tracing_subscriber::{fmt::format, prelude::*};
    use tracing_subscriber::{
        fmt::{FmtContext, FormatEvent, FormatFields, format::Writer},
        registry::LookupSpan,
    };

    use cargo_near_build::env_keys;

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

    if rust_log_env_is_set {
        let environment = if std::env::var(env_keys::nep330::BUILD_ENVIRONMENT).is_ok() {
            "container".cyan()
        } else {
            "host".purple()
        };
        let my_formatter = types::my_formatter::MyFormatter::from_environment(environment);

        let format = format::debug_fn(move |writer, _field, value| write!(writer, "{value:?}"));

        let _e = tracing_subscriber::registry()
            .with(
                tracing_subscriber::fmt::layer()
                    .event_format(my_formatter)
                    .fmt_fields(format)
                    .with_filter(EnvFilter::from_default_env()),
            )
            .try_init();
    } else if teach_me_flag_is_set {
        let env_filter = EnvFilter::from_default_env()
            .add_directive(tracing::Level::WARN.into())
            .add_directive("near_teach_me=info".parse()?)
            .add_directive("near_cli_rs=info".parse()?)
            .add_directive("tracing_instrument=info".parse()?);
        tracing_subscriber::registry()
            .with(tracing_subscriber::fmt::layer().event_format(SimpleFormatter))
            .with(env_filter)
            .init();
    } else {
        let indicatif_layer = IndicatifLayer::new()
            .with_progress_style(
                ProgressStyle::with_template(
                    "{spinner:.blue}{span_child_prefix} {span_name} {msg} {span_fields}",
                )
                .unwrap()
                .tick_strings(&["◐", "◓", "◑", "◒"]),
            )
            .with_span_child_prefix_symbol("↳ ");
        let env_filter = EnvFilter::from_default_env()
            .add_directive(tracing::Level::WARN.into())
            .add_directive("near_cli_rs=info".parse()?)
            .add_directive("tracing_instrument=info".parse()?);
        tracing_subscriber::registry()
            .with(
                tracing_subscriber::fmt::layer()
                    .event_format(SimpleFormatter)
                    .with_writer(indicatif_layer.get_stderr_writer()),
            )
            .with(indicatif_layer)
            .with(env_filter)
            .init();
    };
    Ok(())
}

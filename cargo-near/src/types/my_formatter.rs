use colored::Colorize;
use tracing::Level;
use tracing_core::{Event, Subscriber};
use tracing_log::NormalizeEvent;
use tracing_subscriber::fmt::{
    format::{self, FormatEvent, FormatFields},
    FmtContext,
};
use tracing_subscriber::registry::LookupSpan;

#[derive(Debug)]
pub struct MyFormatter {
    environment: colored::ColoredString,
}

impl MyFormatter {
    pub fn from_environment(environment: colored::ColoredString) -> Self {
        Self { environment }
    }
}

impl<S, N> FormatEvent<S, N> for MyFormatter
where
    S: Subscriber + for<'a> LookupSpan<'a>,
    N: for<'a> FormatFields<'a> + 'static,
{
    fn format_event(
        &self,
        ctx: &FmtContext<'_, S, N>,
        mut writer: format::Writer<'_>,
        event: &Event<'_>,
    ) -> std::fmt::Result {
        let normalized_meta = event.normalized_metadata();
        let metadata = normalized_meta.as_ref().unwrap_or_else(|| event.metadata());

        let level = format!("[{}]", metadata.level());
        let fmt_level = match *metadata.level() {
            Level::ERROR => level.red(),
            Level::WARN => level.yellow(),
            Level::INFO => level.cyan(),
            Level::DEBUG => level.truecolor(100, 100, 100),
            Level::TRACE => level.truecolor(200, 200, 200),
        };

        write!(
            &mut writer,
            "{}-[{}] {}:{} - ",
            fmt_level,
            self.environment,
            metadata.file().unwrap_or("log"),
            metadata.line().unwrap_or_default()
        )?;

        ctx.field_format().format_fields(writer.by_ref(), event)?;

        writeln!(writer)
    }
}

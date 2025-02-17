//! # A `FlexiLoggerView` for cursive
//!
//! This crate provides a new debug view for
//! [gyscos/cursive](https://github.com/gyscos/cursive) using the
//! [emabee/flexi_logger](https://github.com/emabee/flexi_logger) crate. This
//! enables the `FlexiLoggerView` to respect the `RUST_LOG` environment variable
//! as well as the `flexi_logger` configuration file. Have a look at the `demo`
//! below to see how it looks.
//!
//! ## Using the `FlexiLoggerView`
//!
//! To create a `FlexiLoggerView` you first have to register the
//! `boxed_flexi_log_writer(&cursive)` or `CursiveLogWriter::new(&cursive).into_boxed()` as a `LogTarget` in `flexi_logger`. After the
//! `flexi_logger` has started, you may create a `FlexiLoggerView::new().wrap_scroll_view()` instance and
//! add it to cursive.
//!
//! ```rust
//! use cursive::{Cursive, CursiveExt};
//! use cursive_logger_view::{CursiveLogWriter, FlexiLoggerView};
//! use flexi_logger::Logger;
//!     // we need to initialize cursive first, as the cursive-flexi-logger
//!     // needs a cursive callback sink to notify cursive about screen refreshs
//!     // when a new log message arrives
//!     let mut siv = Cursive::default();
//!
//!     // Configure the flexi logger with environment-variable($RUST_LOG) or fallback to "trace" level
//!     Logger::try_with_env_or_str("trace")
//!         .expect("Could not create Logger from environment :(")
//!         // Configure logging to both file and Cursive
//!         .log_to_file_and_writer(
//!             // File configuration: store logs in 'logs' directory without timestamps
//!             flexi_logger::FileSpec::default()
//!                 .directory("logs")
//!                 .suppress_timestamp(),
//!             // Create Cursive log writer and box it for dynamic dispatch
//!             CursiveLogWriter::new(&siv)
//!                 //// Optional format configuration (commented out example)
//!                 // .with_format({
//!                 //     use cursive_logger_view::LogItems::*;
//!                 //     [Level, DateTime, ModLine, Message]
//!                 //         .into_iter()
//!                 //         .collect()
//!                 // })
//!                 // .with_time_format("%T%.6f".into())
//!                 .into_boxed(),
//!         )
//!         .start()
//!         .expect("failed to initialize logger!");
//!
//!     // Add the logger view to Cursive
//!     siv.add_layer(
//!         FlexiLoggerView::new()
//!             // .with_indent(true)  // Optional indentation configuration
//!             //
//!             .wrap_scroll_view(),
//!     );
//!
//!     log::info!("test log message");
//!     // siv.run();
//! ```
//!
//!
//! ```rust
//! use cursive::{Cursive, CursiveExt};
//! use cursive_logger_view::{show_flexi_logger_debug_console, hide_flexi_logger_debug_console, toggle_flexi_logger_debug_console};
//! use flexi_logger::Logger;
//!     // we need to initialize cursive first, as the cursive-flexi-logger
//!     // needs a cursive callback sink to notify cursive about screen refreshs
//!     // when a new log message arrives
//!     let mut siv = Cursive::default();
//!
//!     Logger::try_with_env_or_str("trace")
//!         .expect("Could not create Logger from environment :(")
//!         .log_to_file_and_writer(
//!            flexi_logger::FileSpec::default()
//!                 .directory("logs")
//!                 .suppress_timestamp(),
//!             cursive_logger_view::boxed_flexi_log_writer(&siv)
//!         )
//!         .start()
//!         .expect("failed to initialize logger!");
//!
//!     siv.add_global_callback('~', toggle_flexi_logger_debug_console);  // Bind '~' key to show/hide debug console view
//!     siv.add_global_callback('s', show_flexi_logger_debug_console);  // Bind 's' key to show debug console view
//!     siv.add_global_callback('h', hide_flexi_logger_debug_console);  // Bind 'h' key to hide debug console view
//!
//!     log::info!("test log message");
//!     // siv.run();
//! ```
mod formatter;
pub mod toggle;
mod view;

use arraydeque::{behavior, ArrayDeque};
use compact_str::CompactString;
use cursive_core::{utils::markup::StyledString, CbSink, Cursive};
use flexi_logger::writers::LogWriter;
use getset::WithSetters;
use std::sync::{Arc, Mutex, OnceLock};
use tap::Pipe;
use tinyvec::TinyVec;

type LogBuffer = ArrayDeque<StyledString, 2048, behavior::Wrapping>;

static FLEXI_LOGGER_DEBUG_VIEW_NAME: &str = "_flexi_debug_view";

fn static_logs() -> &'static Arc<Mutex<LogBuffer>> {
    static LOGS: OnceLock<Arc<Mutex<LogBuffer>>> = OnceLock::new();
    LOGS.get_or_init(|| Arc::new(Mutex::new(LogBuffer::new())))
}
const GET_LOCK_ERR_MSG: &str = "Failed to get static_logs Mutex Lock";

/// The `FlexiLoggerView` displays log messages from the `cursive_flexi_logger` log target.
///
/// ```rust
/// use cursive_logger_view::FlexiLoggerView;
///
/// FlexiLoggerView::new().with_indent(true);
/// ```
#[derive(Default, Debug, WithSetters)]
pub struct FlexiLoggerView {
    #[getset(set_with = "pub")]
    pub indent: bool,
}

///Possible log items
#[derive(Debug)]
pub enum LogItems<'c> {
    DateTime,
    Thread,
    ModLine,
    File,
    FileLine,
    Level,
    Message,
    // ThreadLine,
    Custom(&'c str),
}

impl Default for LogItems<'_> {
    fn default() -> Self {
        Self::Level
    }
}

/// The `flexi_logger` `LogWriter` implementation for the `FlexiLoggerView`.
///
/// Use the `boxed_flexi_log_writer` or `CursiveLogWriter::new` function to create an instance of this struct.
#[derive(Debug, WithSetters)]
#[getset(set_with = "pub")]
pub struct CursiveLogWriter<'fmt> {
    sink: CbSink,
    format: TinyVec<[LogItems<'fmt>; 8]>,
    time_format: CompactString,
}

impl CursiveLogWriter<'_> {
    pub fn new(siv: &Cursive) -> Self {
        use crate::LogItems::{DateTime, Level, Message, ModLine};

        Self {
            sink: siv.cb_sink().clone(),
            format: [DateTime, Level, ModLine, Message]
                .into_iter()
                .collect(),
            time_format: "%T%.3f".pipe(CompactString::const_new),
        }
    }

    pub fn into_boxed(self) -> Box<Self> {
        // Box::new(self)
        self.into()
    }
}

/// Creates a new `LogWriter` instance for the `FlexiLoggerView`. Use this to
/// register a cursive log writer in `flexi_logger`.
///
/// Although, it is safe to create multiple cursive log writers, it may not be
/// what you want. Each instance of a cursive log writer replicates the log
/// messages in to `FlexiLoggerView`. When registering multiple cursive log
/// writer instances, a single log messages will be duplicated by each log
/// writer.
///
/// # Registering the cursive log writer in `flexi_logger`
///
/// ```rust
/// use cursive::{Cursive, CursiveExt};
/// use flexi_logger::Logger;
///
///     // we need to initialize cursive first, as the cursive-flexi-logger
///     // needs a cursive callback sink to notify cursive about screen refreshs
///     // when a new log message arrives
///     let mut siv = Cursive::default();
///
///     Logger::try_with_env_or_str("trace")
///         .expect("Could not create Logger from environment :(")
///         .log_to_file_and_writer(
///            flexi_logger::FileSpec::default()
///                 .directory("logs")
///                 .suppress_timestamp(),
///             cursive_logger_view::boxed_flexi_log_writer(&siv)
///         )
///         .start()
///         .expect("failed to initialize logger!");
/// ```
pub fn boxed_flexi_log_writer(siv: &Cursive) -> Box<dyn LogWriter> {
    CursiveLogWriter::new(siv) //
        .pipe(Box::new)
}

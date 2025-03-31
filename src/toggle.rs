//! ## Add toggleable flexi_logger debug console view
//!
//! This crate provides utility functions, which is simplify usage of
//! `FlexiLoggerView`, providing debug console view like
//! [`Cursive::toggle_debug_console`](/cursive/latest/cursive/struct.Cursive.
//! html#method.toggle_debug_console). There is 3 functions:
//!
//!  - `show_flexi_logger_debug_console`: show debug console view;
//!  - `hide_flexi_logger_debug_console`: hide debug console view (if visible);
//!  - `toggle_flexi_logger_debug_console`: show the debug console view, or hide
//!    it if it's already visible.

use cursive_core::{view::Nameable, views::Dialog, Cursive};
use tap::Pipe;

use crate::{FlexiLoggerView, FLEXI_LOGGER_DEBUG_VIEW_NAME};

/// Show the flexi_logger debug console.
///
/// This is analog to
/// [`Cursive::show_debug_console`](/cursive/latest/cursive/struct.Cursive.html#
/// method.show_debug_console).
///
/// # Add binding to show flexi_logger debug view
///
/// ```rust
/// use cursive::{Cursive, CursiveExt};
/// use cursive_logger_view::toggle::show_flexi_logger_debug_console;
/// use flexi_logger::Logger;
///
///     // we need to initialize cursive first, as the cursive-logger-view
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
///
///     siv.add_global_callback('~', show_flexi_logger_debug_console);  // Add binding to show flexi_logger debug view
///
///     // siv.run();
/// ```
pub fn show_flexi_logger_debug_console(siv: &mut Cursive) {
  FlexiLoggerView::new()
    .wrap_scroll_view()
    .with_name(FLEXI_LOGGER_DEBUG_VIEW_NAME)
    .pipe(Dialog::around)
    .title("Debug console")
    .pipe(|v| siv.add_layer(v))
}

/// Hide the flexi_logger debug console (if visible).
///
/// # Add binding to hide flexi_logger debug view
///
/// ```rust
/// use cursive::{Cursive, CursiveExt};
/// use cursive_logger_view::toggle::hide_flexi_logger_debug_console;
/// use flexi_logger::Logger;
///     // we need to initialize cursive first, as the cursive-logger-view
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
///
///     siv.add_global_callback('~', hide_flexi_logger_debug_console);  // Add binding to hide flexi_logger debug view
///
///     // siv.run();
/// ```
pub fn hide_flexi_logger_debug_console(siv: &mut Cursive) {
  if let Some(pos) = siv
    .screen_mut()
    .find_layer_from_name(FLEXI_LOGGER_DEBUG_VIEW_NAME)
  {
    siv
      .screen_mut()
      .remove_layer(pos);
  }
}

/// Show the flexi_logger debug console, or hide it if it's already visible.
///
/// This is analog to
/// [`Cursive::toggle_debug_console`](/cursive/latest/cursive/struct.Cursive.
/// html#method.toggle_debug_console).
///
/// # Enable toggleable flexi_logger debug view
///
/// ```rust
/// use cursive::{Cursive, CursiveExt};
/// use cursive_logger_view::toggle::toggle_flexi_logger_debug_console;
/// use flexi_logger::Logger;
///     // we need to initialize cursive first, as the cursive-logger-view
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
///
///     siv.add_global_callback('~', toggle_flexi_logger_debug_console);  // Enable toggleable flexi_logger debug view
///
///     // siv.run();
/// ```
pub fn toggle_flexi_logger_debug_console(siv: &mut Cursive) {
  match siv
    .screen_mut()
    .find_layer_from_name(FLEXI_LOGGER_DEBUG_VIEW_NAME)
  {
    Some(pos) => {
      siv
        .screen_mut()
        .remove_layer(pos);
    }
    _ => show_flexi_logger_debug_console(siv),
  }
}

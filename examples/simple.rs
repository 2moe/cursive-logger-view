use std::time::Duration;

use cursive::{view::Resizable, views::Dialog, Cursive, CursiveExt, Vec2};
use cursive_logger_view::FlexiLoggerView;

fn main() {
  // we need to initialize cursive first, as the cursive-logger-view
  // needs a cursive callback sink to notify cursive about screen refreshs
  // when a new log message arrives
  let mut siv = Cursive::default();

  flexi_logger::Logger::try_with_env_or_str("trace")
    .expect("Could not create Logger from environment :(")
    .log_to_file_and_writer(
      flexi_logger::FileSpec::default()
        .directory("logs")
        .suppress_timestamp(),
      cursive_logger_view::boxed_flexi_log_writer(&siv),
    )
    .start()
    .expect("failed to initialize logger!");

  siv.add_layer(
    Dialog::around(FlexiLoggerView::new().wrap_scroll_view())
      .title("Flexi-Logger View")
      .button("Quit", |siv| siv.quit())
      .fixed_size(Vec2::new(72, 13)),
  );

  log::info!("started simple example");

  let sink = siv.cb_sink().clone();
  std::thread::Builder::new()
    .name("worker".into())
    .spawn(move || {
      log::trace!("A trace log message");
      std::thread::sleep(Duration::from_secs(1));

      log::debug!("A debug log message");
      std::thread::sleep(Duration::from_secs(1));

      log::info!("An info log message");
      std::thread::sleep(Duration::from_secs(1));

      log::debug!("Really detailed debug information\nfoo: 5\nbar: 42");
      std::thread::sleep(Duration::from_secs(1));

      log::warn!("A warning log message");
      std::thread::sleep(Duration::from_secs(1));

      log::error!("An error log message");
      std::thread::sleep(Duration::from_secs(1));

      sink
        .send(Box::new(|siv| siv.quit()))
        .expect("failed to quit");
    })
    .expect("worker thread panicked!");

  siv.run();
}

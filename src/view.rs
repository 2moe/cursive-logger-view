use crate::{static_logs, FlexiLoggerView, GET_LOCK_ERR_MSG};
use cursive_core::{
    view::{ScrollStrategy, Scrollable, View},
    views::ScrollView,
    Printer, Vec2,
};
use tap::Pipe;
use unicode_width::UnicodeWidthStr;

impl FlexiLoggerView {
    /// Wraps a `FlexiLoggerView` in a `ScrollView`.
    ///
    /// # Example
    ///
    /// ```
    /// use cursive_logger_view::FlexiLoggerView;
    ///
    /// FlexiLoggerView::new()
    ///     .with_indent(false)
    ///     .wrap_scroll_view();
    /// ```
    pub fn wrap_scroll_view(self) -> ScrollView<Self> {
        self.scrollable()
            .scroll_x(true)
            .scroll_y(true)
            .scroll_strategy(ScrollStrategy::StickToBottom)
    }

    /// Creates a new `FlexiLoggerView`.
    pub fn new() -> Self {
        FlexiLoggerView { indent: true }
    }
}

impl View for FlexiLoggerView {
    fn draw(&self, printer: &Printer<'_, '_>) {
        let logs = static_logs()
            .lock()
            .expect(GET_LOCK_ERR_MSG);

        // Only print the last logs, so skip what doesn't fit
        let skipped = logs
            .len()
            .saturating_sub(printer.size.y);

        logs.iter()
            .skip(skipped)
            .fold(0, |y, msg| {
                let log_msg_index = msg.spans_raw().len() - 1;

                let x = msg
                    .spans()
                    .take(log_msg_index)
                    .fold(0, |x, span| {
                        printer.with_style(*span.attr, |p| {
                            p.print((x, y), span.content)
                        });
                        x + span.width
                    });

                msg.spans()
                    .nth(log_msg_index)
                    .map(|log_msg| {
                        log_msg
                            .content
                            .split('\n')
                            .enumerate()
                            .fold(y, |current_y, (i, part)| {
                                let x_pos =
                                    if !self.indent && i > 0 { 0 } else { x };
                                printer.with_style(*log_msg.attr, |p| {
                                    p.print((x_pos, current_y), part)
                                });
                                current_y + 1
                            })
                    })
                    .unwrap_or(y)
            });
    }

    fn required_size(&mut self, constraint: Vec2) -> Vec2 {
        let logs = static_logs()
            .lock()
            .expect(GET_LOCK_ERR_MSG);

        // The longest line sets the width
        let width = logs
            .iter()
            .map(|msg| {
                msg.spans()
                    .map(|x|
                    // if the log message contains more than one line,
                    // only the longest line should be considered
                    // (definitely not the total content.len())
                    x.content.split('\n').map(|x| x.width()).max().expect("ERR: required_size(), failed to get width"))
                    .sum::<usize>()
            })
            .max()
            .unwrap_or(1)
            .pipe(|w|core::cmp::max(w, constraint.x));

        let height = logs
            .iter()
            .map(|msg| {
                msg.spans()
                    .last()
                    .map(|x| x.content.split('\n').count())
                    .expect("ERR: required_size(), the last span message is invalid, and failed to get height.")
            })
            .sum::<usize>()
            .pipe(|h| core::cmp::max(h, constraint.y));

        Vec2::new(width, height)
    }
}

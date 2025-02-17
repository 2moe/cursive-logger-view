use crate::{log_buffer, CursiveLogWriter, LogItems, GET_LOCK_ERR_MSG};
use compact_str::{format_compact, CompactString, ToCompactString};
use cursive_core::{
    theme::{BaseColor, Color},
    utils::markup::StyledString,
};
use flexi_logger::{writers::LogWriter, DeferredNow, Record};
use getset::WithSetters;
use log::Level;
use std::{io, thread};
use tap::Pipe;

const fn log_level_as_dark_color(level: &Level) -> Color {
    use BaseColor::{Cyan, Green, Magenta, Red, Yellow};
    use Level::*;
    let base_color = match level {
        Trace => Magenta,
        Debug => Cyan,
        Info => Green,
        Warn => Yellow,
        Error => Red,
    };
    Color::Dark(base_color)
}

#[derive(Debug, WithSetters)]
struct StyledTextConfig<'a> {
    line: &'a mut StyledString,
    #[getset(set_with)]
    content: CompactString,
    #[getset(set_with)]
    color_enabled: bool,
    color: Color,
}

impl StyledTextConfig<'_> {
    fn append_line(mut self) -> Self {
        let content = (&mut self.content) //
            .pipe(core::mem::take);

        match self.color_enabled {
            true => self
                .line
                .append_styled(content, self.color),
            _ => self.line.append_plain(content),
        }
        self
    }

    fn append_mod_line(self, rec: &Record) -> Self {
        let (path, line_num) = (
            rec.module_path().unwrap_or(""), //
            rec.line().unwrap_or(0),
        );

        format_compact!("<{path}:") //
            .pipe(|s| self.line.append_plain(s));

        format_compact!("{line_num}") //
            .pipe(|s| {
                self.line
                    .append_styled(s, Color::Dark(BaseColor::Blue))
            });

        self.line.append_plain("> ");

        self
    }
}

impl LogWriter for CursiveLogWriter<'_> {
    fn write(&self, now: &mut DeferredNow, record: &Record) -> io::Result<()> {
        let styled_config = StyledTextConfig {
            line: &mut StyledString::new(),
            content: "".into(),
            color_enabled: false,
            color: record
                .level()
                .pipe_ref(log_level_as_dark_color),
        };

        let line = self
            .format
            .iter()
            .fold(styled_config, |cfg, item| {
                use crate::LogItems::{
                    DateTime, File, FileLine, Level, Message, ModLine, Thread,
                };

                match item {
                    DateTime => now
                        .format(&self.time_format)
                        .pipe(|fmt| format_compact!("{fmt} "))
                        .pipe(|s| cfg.with_content(s))
                        .with_color_enabled(false)
                        .append_line(),
                    //
                    Thread => thread::current()
                        .name()
                        .unwrap_or(" ")
                        .pipe(|name| format_compact!("[{name}] "))
                        .pipe(|s| cfg.with_content(s))
                        .with_color_enabled(false)
                        .append_line(),
                    //
                    Level => record
                        .level()
                        .pipe(|lv| format_compact!("[{lv}] "))
                        .pipe(|s| cfg.with_content(s))
                        .with_color_enabled(true)
                        .append_line(),
                    //
                    File => record
                        .file()
                        .unwrap_or("")
                        .pipe(|file| format_compact!("<{file}> "))
                        .pipe(|s| cfg.with_content(s))
                        .with_color_enabled(false)
                        .append_line(),
                    //
                    FileLine => format_compact!(
                        "<{}:{}> ",
                        record.file().unwrap_or(""),
                        record.line().unwrap_or(0),
                    )
                    .pipe(|s| cfg.with_content(s))
                    .with_color_enabled(false)
                    .append_line(),
                    //
                    ModLine => cfg.append_mod_line(record),
                    //
                    Message => record
                        .args()
                        .pipe(|x| format_compact!("{x}"))
                        .pipe(|s| cfg.with_content(s))
                        .with_color_enabled(true)
                        .append_line(),
                    //
                    LogItems::Custom(txt) => cfg
                        .with_content(txt.to_compact_string())
                        .with_color_enabled(true)
                        .append_line(), // line.append_plain(txt.as_str()),
                }
            })
            .line
            .pipe(core::mem::take);

        log_buffer::static_logs()
            .lock()
            .expect(GET_LOCK_ERR_MSG)
            .push_back(line);

        let io_broken_pipe = |msg| io::Error::new(io::ErrorKind::BrokenPipe, msg);

        self.sink
            .send(Box::new(|_| {}))
            .map_err(|_| io_broken_pipe("cursive callback sink is closed!"))
    }

    fn flush(&self) -> io::Result<()> {
        // we are not buffering
        Ok(())
    }

    fn max_log_level(&self) -> log::LevelFilter {
        log::LevelFilter::max()
    }
}

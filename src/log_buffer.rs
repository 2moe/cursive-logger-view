use arraydeque::{behavior, ArrayDeque};
use cursive_core::utils::markup::StyledString;
use std::sync::{Arc, Mutex, OnceLock};
use tap::Pipe;

type LogBuffer = ArrayDeque<StyledString, 2048, behavior::Wrapping>;
type ArcMutexBuffer = Arc<Mutex<LogBuffer>>;

pub(crate) fn static_logs() -> &'static ArcMutexBuffer {
    static LOGS: OnceLock<ArcMutexBuffer> = OnceLock::new();
    LOGS.get_or_init(|| {
        LogBuffer::new()
            .pipe(Mutex::new)
            .pipe(Arc::new)
    })
}

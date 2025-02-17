use arraydeque::{behavior, ArrayDeque};
use cursive_core::utils::markup::StyledString;
use std::sync::{Arc, Mutex, OnceLock};
use tap::Pipe;

pub(crate) const GET_LOCK_ERR_MSG: &str = "Failed to get static_logs Mutex Lock";

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

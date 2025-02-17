use circular_buffer::CircularBuffer;
use cursive_core::utils::markup::StyledString;
use std::sync::{Arc, Mutex, OnceLock};
use tap::Pipe;

/// Pre-defined error message for lock acquisition failures
pub(crate) const GET_LOCK_ERR_MSG: &str = "Failed to get static_logs Mutex Lock";

/// Type alias for a ring buffer with fixed capacity storing styled strings.
///
/// Why CircularBuffer?
/// 1. Provides efficient FIFO (a.k.a. First In First Out) behavior with fixed capacity.
/// 2. Automatically overwriting oldest entries when full.
///
/// ⚠️ Important Note
/// - Ensure you call `LogBuffer::boxed()` instead of `LogBuffer::new()` for initialization.
///   - Reason: This data-structure is very large. If allocated on the stack via `new()`, it could cause a stack overflow.
type LogBuffer = CircularBuffer<3072, StyledString>;

/// Thread-safe shared buffer type breakdown:
/// - Box: Ensures buffer allocation stays on the heap
/// - Mutex: Provides exclusive access synchronization
/// - Arc: Enables shared ownership across threads
type ArcMutexBuffer = Arc<Mutex<Box<LogBuffer>>>;

/// Initializes and provides global access to the thread-safe log buffer
///
/// Why OnceLock? Ensures:
/// 1. Thread-safe lazy initialization
/// 2. Avoids "double initialization" race conditions
/// 3. No unnecessary allocation before first use
pub(crate) fn static_logs() -> &'static ArcMutexBuffer {
    static LOGS: OnceLock<ArcMutexBuffer> = OnceLock::new();

    LOGS.get_or_init(|| {
        // Critical initialization sequence:
        // 1. Create heap-allocated buffer (Box prevents stack overflow)
        // 2. Wrap in Mutex for thread-safe access
        // 3. Share via Arc for cross-thread cloning
        LogBuffer::boxed() // Explicit heap allocation
            .pipe(Mutex::new) // Add synchronization layer
            .pipe(Arc::new) // Enable shared ownership
    })
}

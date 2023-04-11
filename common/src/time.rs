use std::fmt::Debug;
#[derive(Copy, Clone)]
pub struct Duration {
    millis: f64,
}
impl Duration {
    /// Represents the time in milliseconds.
    pub fn as_millis(&self) -> f64 {
        self.millis
    }
    /// Represents the time in seconds.
    pub fn as_secs(&self) -> f64 {
        self.millis / 1000.0
    }
    pub fn as_secs_f32(&self) -> f32 {
        self.millis as f32 / 1000.0
    }
    /// Represents the time as nanoseconds.
    pub fn as_nanos(&self) -> f64 {
        self.millis * 1000.0
    }

    pub fn new(millis: f64) -> Self {
        Self { millis }
    }
    pub fn from_secs(secs: f64) -> Self {
        Self {
            millis: secs * 1000.,
        }
    }
}
impl PartialEq for Duration {
    fn eq(&self, other: &Self) -> bool {
        self.millis == other.millis
    }
}
impl PartialOrd for Duration {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.millis.partial_cmp(&other.millis)
    }
}
impl Debug for Duration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}ms", self.millis))
    }
}

/// Platform-agnostic timer struct designed as a replacement for std::time.
pub struct Timer {
    #[cfg(target_arch = "wasm32")]
    start_time: f64,
    #[cfg(not(target_arch = "wasm32"))]
    start_time: std::time::Instant,
}

#[cfg(target_arch = "wasm32")]
fn now() -> f64 {
    web_sys::window()
        .expect("Unable to get window!")
        .performance()
        .expect("Performance object not available.")
        .now()
}
#[cfg(target_arch = "wasm32")]
impl Timer {
    /// Creates a new timer and immediately starts it.
    pub fn new() -> Self {
        Self { start_time: now() }
    }
    /// Starts or restarts the timer
    pub fn start(&mut self) {
        self.start_time = now();
    }
    /// Gets the elapsed time, in milliseconds.
    pub fn elapsed(&mut self) -> Duration {
        Duration {
            millis: now() - self.start_time,
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl Timer {
    /// Creates a new timer and immediately starts it.
    pub fn new() -> Self {
        Self {
            start_time: std::time::Instant::now(),
        }
    }
    /// Starts or restarts the timer
    pub fn start(&mut self) {
        self.start_time = std::time::Instant::now();
    }
    /// Gets the elapsed time, in milliseconds.
    pub fn elapsed(&mut self) -> Duration {
        Duration {
            millis: (std::time::Instant::now() - self.start_time).as_secs_f64() * 1000.0,
        }
    }
}

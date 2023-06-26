use std::time::Duration;

#[doc(hidden)]
#[derive(Debug, Clone, Copy)]
pub struct TimerChange {
    pub(super) timestamp: Duration
}

#[doc(hidden)]
impl From<Duration> for TimerChange {
    fn from(timer_change: Duration) -> Self {
        Self { timestamp: timer_change }
    }
}

#[doc(hidden)]
impl From<TimerChange> for Duration {
    fn from(timer_change: TimerChange) -> Self {
        timer_change.timestamp
    }
}

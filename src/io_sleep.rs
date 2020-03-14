use futures_timer::Delay;
use std::time::Duration;

pub async fn io_sleep_ms(n: u64) {
    Delay::new(Duration::from_millis(n)).await
}

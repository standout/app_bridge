use std::time::{SystemTime, UNIX_EPOCH};

pub fn timestamp() -> u64 {
  let now = SystemTime::now();
  let timestamp = now.duration_since(UNIX_EPOCH)
      .expect("Time went backwards")
      .as_millis() as u64;

  timestamp
}

use std::collections::VecDeque;
use std::time::{Duration, Instant};

pub struct FrameRateTracker {
   last: Instant,
   pub history: VecDeque<f64>,
   pub cap: usize,
}
impl FrameRateTracker {
   pub fn start(cap: usize) -> Self {
      Self {
         last: Instant::now(),
         history: VecDeque::with_capacity(cap),
         cap,
      }
   }

   pub fn update(&mut self) {
      self.history.push_front(self.last.elapsed().as_secs_f64());
      if self.history.len() > self.cap {
         self.history.pop_back();
      }
      self.last = Instant::now();
   }

   pub fn get_frametime(&self) -> Duration {
      Duration::from_secs_f64(self.history.iter().sum::<f64>() / self.history.len() as f64)
   }
}
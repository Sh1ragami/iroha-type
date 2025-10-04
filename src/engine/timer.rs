use std::time::{Instant, Duration};

pub struct Timer { start: Instant }
impl Timer { pub fn new() -> Self { Self{ start: Instant::now() } } pub fn elapsed(&self)->Duration{ self.start.elapsed() } }


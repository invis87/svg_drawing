pub struct TickTimer {
    pub time: f64,
}

impl Default for TickTimer {
    fn default() -> Self {
        TickTimer { time: 0.0 }
    }
}

impl TickTimer {
    const TICK_PERIOD: f64 = 0.001; //todo: number of ticks should be calculated based on curve length
}

impl Iterator for TickTimer {
    type Item = f64;

    fn next(&mut self) -> Option<Self::Item> {
        if self.time > 1.0 {
            None
        } else {
            let current_value = self.time;
            self.time += TickTimer::TICK_PERIOD;
            Some(current_value)
        }
    }
}

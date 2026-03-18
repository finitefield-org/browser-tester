use super::*;

/// Trace and determinism controls for a [`Harness`].
impl Harness {
    /// Enable or disable trace log emission to stderr.
    pub fn set_trace_stderr(&mut self, enabled: bool) {
        self.trace_state.to_stderr = enabled;
    }

    /// Enable or disable event trace collection.
    pub fn set_trace_events(&mut self, enabled: bool) {
        self.trace_state.events = enabled;
    }

    /// Enable or disable timer trace collection.
    pub fn set_trace_timers(&mut self, enabled: bool) {
        self.trace_state.timers = enabled;
    }

    /// Set the maximum number of retained trace log entries.
    pub fn set_trace_log_limit(&mut self, max_entries: usize) -> Result<()> {
        if max_entries == 0 {
            return Err(Error::ScriptRuntime(
                "set_trace_log_limit requires at least 1 entry".into(),
            ));
        }
        self.trace_state.log_limit = max_entries;
        while self.trace_state.logs.len() > self.trace_state.log_limit {
            self.trace_state.logs.pop_front();
        }
        Ok(())
    }

    /// Seed the deterministic random number generator.
    pub fn set_random_seed(&mut self, seed: u64) {
        self.rng_state = if seed == 0 {
            0xA5A5_A5A5_A5A5_A5A5
        } else {
            seed
        };
    }
}

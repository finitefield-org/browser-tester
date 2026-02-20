use super::*;

impl Harness {
    pub fn now_ms(&self) -> i64 {
        self.scheduler.now_ms
    }

    pub fn clear_timer(&mut self, timer_id: i64) -> bool {
        let existed = self.scheduler.running_timer_id == Some(timer_id)
            || self
                .scheduler
                .task_queue
                .iter()
                .any(|task| task.id == timer_id);
        self.clear_timeout(timer_id);
        existed
    }

    pub fn clear_all_timers(&mut self) -> usize {
        let cleared = self.scheduler.task_queue.len();
        self.scheduler.task_queue.clear();
        if self.scheduler.running_timer_id.is_some() {
            self.scheduler.running_timer_canceled = true;
        }
        self.trace_timer_line(format!("[timer] clear_all cleared={cleared}"));
        cleared
    }

    pub fn pending_timers(&self) -> Vec<PendingTimer> {
        let mut timers = self
            .scheduler
            .task_queue
            .iter()
            .map(|task| PendingTimer {
                id: task.id,
                due_at: task.due_at,
                order: task.order,
                interval_ms: task.interval_ms,
            })
            .collect::<Vec<_>>();
        timers.sort_by_key(|timer| (timer.due_at, timer.order));
        timers
    }

    pub fn advance_time(&mut self, delta_ms: i64) -> Result<()> {
        if delta_ms < 0 {
            return Err(Error::ScriptRuntime(
                "advance_time requires non-negative milliseconds".into(),
            ));
        }
        let from = self.scheduler.now_ms;
        self.scheduler.now_ms = self.scheduler.now_ms.saturating_add(delta_ms);
        let ran = self.run_due_timers_internal()?;
        self.trace_timer_line(format!(
            "[timer] advance delta_ms={} from={} to={} ran_due={}",
            delta_ms, from, self.scheduler.now_ms, ran
        ));
        Ok(())
    }

    pub fn advance_time_to(&mut self, target_ms: i64) -> Result<()> {
        if target_ms < self.scheduler.now_ms {
            return Err(Error::ScriptRuntime(format!(
                "advance_time_to requires target >= now_ms (target={target_ms}, now_ms={})",
                self.scheduler.now_ms
            )));
        }
        let from = self.scheduler.now_ms;
        self.scheduler.now_ms = target_ms;
        let ran = self.run_due_timers_internal()?;
        self.trace_timer_line(format!(
            "[timer] advance_to from={} to={} ran_due={}",
            from, self.scheduler.now_ms, ran
        ));
        Ok(())
    }

    pub fn flush(&mut self) -> Result<()> {
        let from = self.scheduler.now_ms;
        let ran = self.run_timer_queue(None, true)?;
        self.trace_timer_line(format!(
            "[timer] flush from={} to={} ran={}",
            from, self.scheduler.now_ms, ran
        ));
        Ok(())
    }

    pub fn run_next_timer(&mut self) -> Result<bool> {
        let Some(next_idx) = self.next_task_index(None) else {
            self.trace_timer_line("[timer] run_next none".into());
            return Ok(false);
        };

        let task = self.scheduler.task_queue.remove(next_idx);
        if task.due_at > self.scheduler.now_ms {
            self.scheduler.now_ms = task.due_at;
        }
        self.execute_timer_task(task)?;
        Ok(true)
    }

    pub fn run_next_due_timer(&mut self) -> Result<bool> {
        let Some(next_idx) = self.next_task_index(Some(self.scheduler.now_ms)) else {
            self.trace_timer_line("[timer] run_next_due none".into());
            return Ok(false);
        };

        let task = self.scheduler.task_queue.remove(next_idx);
        self.execute_timer_task(task)?;
        Ok(true)
    }

    pub fn run_due_timers(&mut self) -> Result<usize> {
        let ran = self.run_due_timers_internal()?;
        self.trace_timer_line(format!(
            "[timer] run_due now_ms={} ran={}",
            self.scheduler.now_ms, ran
        ));
        Ok(ran)
    }

    pub(crate) fn run_due_timers_internal(&mut self) -> Result<usize> {
        self.run_timer_queue(Some(self.scheduler.now_ms), false)
    }

    pub(crate) fn run_timer_queue(
        &mut self,
        due_limit: Option<i64>,
        advance_clock: bool,
    ) -> Result<usize> {
        let mut steps = 0usize;
        while let Some(next_idx) = self.next_task_index(due_limit) {
            steps += 1;
            if steps > self.scheduler.timer_step_limit {
                return Err(self.timer_step_limit_error(
                    self.scheduler.timer_step_limit,
                    steps,
                    due_limit,
                ));
            }
            let task = self.scheduler.task_queue.remove(next_idx);
            if advance_clock && task.due_at > self.scheduler.now_ms {
                self.scheduler.now_ms = task.due_at;
            }
            self.execute_timer_task(task)?;
        }
        Ok(steps)
    }

    pub(crate) fn timer_step_limit_error(
        &self,
        max_steps: usize,
        steps: usize,
        due_limit: Option<i64>,
    ) -> Error {
        let due_limit_desc = due_limit
            .map(|value| value.to_string())
            .unwrap_or_else(|| "none".into());

        let next_task_desc = self
            .next_task_index(due_limit)
            .and_then(|idx| self.scheduler.task_queue.get(idx))
            .map(|task| {
                let interval_desc = task
                    .interval_ms
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "none".into());
                format!(
                    "id={},due_at={},order={},interval_ms={}",
                    task.id, task.due_at, task.order, interval_desc
                )
            })
            .unwrap_or_else(|| "none".into());

        Error::ScriptRuntime(format!(
            "flush exceeded max task steps (possible uncleared setInterval): limit={max_steps}, steps={steps}, now_ms={}, due_limit={}, pending_tasks={}, next_task={}",
            self.scheduler.now_ms,
            due_limit_desc,
            self.scheduler.task_queue.len(),
            next_task_desc
        ))
    }

    pub(crate) fn next_task_index(&self, due_limit: Option<i64>) -> Option<usize> {
        self.scheduler
            .task_queue
            .iter()
            .enumerate()
            .filter(|(_, task)| {
                if let Some(limit) = due_limit {
                    task.due_at <= limit
                } else {
                    true
                }
            })
            .min_by_key(|(_, task)| (task.due_at, task.order))
            .map(|(idx, _)| idx)
    }

    pub(crate) fn execute_timer_task(&mut self, task: ScheduledTask) -> Result<()> {
        stacker::grow(32 * 1024 * 1024, || self.execute_timer_task_impl(task))
    }

    pub(crate) fn execute_timer_task_impl(&mut self, mut task: ScheduledTask) -> Result<()> {
        let interval_desc = task
            .interval_ms
            .map(|value| value.to_string())
            .unwrap_or_else(|| "none".into());
        self.trace_timer_line(format!(
            "[timer] run id={} due_at={} interval_ms={} now_ms={}",
            task.id, task.due_at, interval_desc, self.scheduler.now_ms
        ));

        self.scheduler.running_timer_id = Some(task.id);
        self.scheduler.running_timer_canceled = false;
        let mut event = EventState::new("timeout", self.dom.root, self.scheduler.now_ms);
        self.run_in_task_context(|this| {
            this.execute_timer_task_callback(
                &task.callback,
                &task.callback_args,
                &mut event,
                &mut task.env,
            )
            .map(|_| ())
        })?;
        let canceled = self.scheduler.running_timer_canceled;
        self.scheduler.running_timer_id = None;
        self.scheduler.running_timer_canceled = false;

        if let Some(interval_ms) = task.interval_ms {
            if !canceled {
                let delay_ms = interval_ms.max(0);
                let due_at = task.due_at.saturating_add(delay_ms);
                let order = self.scheduler.allocate_task_order();
                self.scheduler.task_queue.push(ScheduledTask {
                    id: task.id,
                    due_at,
                    order,
                    interval_ms: Some(delay_ms),
                    callback: task.callback,
                    callback_args: task.callback_args,
                    env: task.env,
                });
                self.trace_timer_line(format!(
                    "[timer] requeue id={} due_at={} interval_ms={}",
                    task.id, due_at, delay_ms
                ));
            }
        }

        Ok(())
    }
}

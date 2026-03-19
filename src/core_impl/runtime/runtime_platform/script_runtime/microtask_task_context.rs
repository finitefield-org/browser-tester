use super::*;

#[path = "microtask_task_context/microtask_callback_context.rs"]
mod microtask_callback_context;
#[path = "microtask_task_context/microtask_listener_capture.rs"]
mod microtask_listener_capture;

impl Harness {
    pub(crate) fn queue_microtask(&mut self, handler: ScriptHandler, env: &HashMap<String, Value>) {
        self.scheduler
            .microtask_queue
            .push_back(ScheduledMicrotask::Script {
                handler,
                env: ScriptEnv::from_snapshot(env),
            });
    }

    pub(crate) fn queue_promise_reaction_microtask(
        &mut self,
        reaction: PromiseReactionKind,
        settled: PromiseSettledValue,
    ) {
        self.scheduler
            .microtask_queue
            .push_back(ScheduledMicrotask::Promise { reaction, settled });
    }

    pub(crate) fn queue_callable_microtask(&mut self, callback: Value) {
        self.scheduler
            .microtask_queue
            .push_back(ScheduledMicrotask::Callable { callback });
    }

    pub(crate) fn queue_worker_message_microtask(
        &mut self,
        worker: &Rc<RefCell<ObjectValue>>,
        target: &Rc<RefCell<ObjectValue>>,
        target_this: Value,
        data: Value,
    ) {
        self.scheduler
            .microtask_queue
            .push_back(ScheduledMicrotask::WorkerMessage {
                worker: worker.clone(),
                target: target.clone(),
                target_this,
                data,
            });
    }

    pub(crate) fn run_microtask_queue(&mut self) -> Result<usize> {
        self.with_task_depth(|this| {
            let mut steps = 0usize;
            loop {
                let Some(task) = this.scheduler.microtask_queue.pop_front() else {
                    return Ok(steps);
                };
                steps += 1;
                if steps > this.scheduler.timer_step_limit {
                    return Err(this.timer_step_limit_error(
                        this.scheduler.timer_step_limit,
                        steps,
                        Some(this.scheduler.now_ms),
                    ));
                }

                match task {
                    ScheduledMicrotask::Script { handler, mut env } => {
                        this.run_script_microtask_handler(&handler, &mut env)?;
                    }
                    ScheduledMicrotask::Callable { callback } => {
                        this.run_callable_microtask(&callback)?;
                    }
                    ScheduledMicrotask::WorkerMessage {
                        worker,
                        target,
                        target_this,
                        data,
                    } => {
                        this.run_worker_message_microtask(&worker, &target, target_this, data)?;
                    }
                    ScheduledMicrotask::Promise { reaction, settled } => {
                        this.run_promise_reaction_task(reaction, settled)?;
                    }
                }
            }
        })
    }

    fn with_task_depth<T>(&mut self, run: impl FnOnce(&mut Self) -> Result<T>) -> Result<T> {
        self.scheduler.task_depth += 1;
        let run_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| run(self)));
        self.scheduler.task_depth = self.scheduler.task_depth.saturating_sub(1);
        match run_result {
            Ok(result) => result,
            Err(payload) => std::panic::resume_unwind(payload),
        }
    }

    pub(crate) fn run_in_task_context<T>(
        &mut self,
        mut run: impl FnMut(&mut Self) -> Result<T>,
    ) -> Result<T> {
        let result = self.with_task_depth(|this| run(this));
        let should_flush_microtasks = self.scheduler.task_depth == 0;
        match result {
            Ok(value) => {
                if should_flush_microtasks {
                    self.run_microtask_queue()?;
                }
                Ok(value)
            }
            Err(err) => Err(err),
        }
    }
}

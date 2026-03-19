use bt_script::{HostBindings, ScriptRuntime};

#[derive(Default)]
struct NoopHost {
    microtasks: usize,
}

impl HostBindings for NoopHost {
    fn on_microtask_checkpoint(&mut self) -> bt_script::Result<()> {
        self.microtasks += 1;
        Ok(())
    }
}

#[test]
fn runtime_tracks_microtask_queue_depth() {
    let mut runtime = ScriptRuntime::new();
    let mut host = NoopHost::default();
    runtime.queue_microtask();
    runtime.queue_microtask();
    runtime
        .run_microtasks(&mut host)
        .expect("runtime should drain microtasks");
    assert_eq!(host.microtasks, 2);
    assert_eq!(runtime.queued_microtasks(), 0);
}

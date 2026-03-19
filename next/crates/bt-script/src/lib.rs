use std::error::Error as StdError;
use std::fmt;

#[derive(Clone, Debug, Default)]
pub struct ScriptParser;

#[derive(Clone, Debug, Default)]
pub struct Evaluator;

#[derive(Clone, Debug, Default)]
pub struct ScriptHeap;

#[derive(Clone, Debug, Default)]
pub struct GlobalEnvironment;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScriptError {
    message: String,
}

impl ScriptError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    pub fn phase_not_ready(capability: &str) -> Self {
        Self::new(format!(
            "{capability} is planned for a later phase of browser-tester-next"
        ))
    }
}

impl fmt::Display for ScriptError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl StdError for ScriptError {}

pub type Result<T> = std::result::Result<T, ScriptError>;

pub trait HostBindings {
    fn on_eval(&mut self, _code: &str, _source_name: &str) -> Result<()> {
        Ok(())
    }

    fn on_microtask_checkpoint(&mut self) -> Result<()> {
        Ok(())
    }
}

#[derive(Clone, Debug, Default)]
pub struct ScriptRuntime {
    parser: ScriptParser,
    evaluator: Evaluator,
    heap: ScriptHeap,
    globals: GlobalEnvironment,
    queued_microtasks: usize,
}

impl ScriptRuntime {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn parser(&self) -> &ScriptParser {
        &self.parser
    }

    pub fn evaluator(&self) -> &Evaluator {
        &self.evaluator
    }

    pub fn heap(&self) -> &ScriptHeap {
        &self.heap
    }

    pub fn globals(&self) -> &GlobalEnvironment {
        &self.globals
    }

    pub fn eval_program<H: HostBindings>(
        &mut self,
        code: &str,
        source_name: &str,
        host: &mut H,
    ) -> Result<()> {
        host.on_eval(code, source_name)
    }

    pub fn queue_microtask(&mut self) {
        self.queued_microtasks += 1;
    }

    pub fn queued_microtasks(&self) -> usize {
        self.queued_microtasks
    }

    pub fn run_microtasks<H: HostBindings>(&mut self, host: &mut H) -> Result<()> {
        while self.queued_microtasks > 0 {
            self.queued_microtasks -= 1;
            host.on_microtask_checkpoint()?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{HostBindings, ScriptRuntime};

    #[derive(Default)]
    struct RecordingHost {
        evals: Vec<(String, String)>,
        microtask_ticks: usize,
    }

    impl HostBindings for RecordingHost {
        fn on_eval(&mut self, code: &str, source_name: &str) -> super::Result<()> {
            self.evals.push((source_name.to_owned(), code.to_owned()));
            Ok(())
        }

        fn on_microtask_checkpoint(&mut self) -> super::Result<()> {
            self.microtask_ticks += 1;
            Ok(())
        }
    }

    #[test]
    fn eval_program_delegates_to_host_bindings() {
        let mut runtime = ScriptRuntime::new();
        let mut host = RecordingHost::default();
        runtime
            .eval_program("document.title = 'x'", "inline-script", &mut host)
            .expect("host callback should succeed");
        assert_eq!(
            host.evals,
            vec![(
                "inline-script".to_string(),
                "document.title = 'x'".to_string(),
            )]
        );
    }

    #[test]
    fn queued_microtasks_are_drained_in_order() {
        let mut runtime = ScriptRuntime::new();
        let mut host = RecordingHost::default();
        runtime.queue_microtask();
        runtime.queue_microtask();
        runtime
            .run_microtasks(&mut host)
            .expect("microtasks should drain");
        assert_eq!(runtime.queued_microtasks(), 0);
        assert_eq!(host.microtask_ticks, 2);
    }
}

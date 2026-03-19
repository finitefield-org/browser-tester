use std::cell::RefCell;
use std::collections::BTreeMap;
use std::error::Error as StdError;
use std::fmt;
use std::rc::Rc;

mod evaluator;
mod parser;
mod syntax;

#[derive(Clone, Debug, Default)]
pub struct ScriptParser;

#[derive(Clone, Debug, Default)]
pub struct Evaluator;

#[derive(Clone, Debug, Default)]
pub struct ScriptHeap;

#[derive(Clone, Debug, Default)]
pub struct GlobalEnvironment;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ElementHandle(u64);

impl ElementHandle {
    pub const fn new(raw: u64) -> Self {
        Self(raw)
    }

    pub const fn raw(self) -> u64 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ListenerTarget {
    Window,
    Document,
    Element(ElementHandle),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum EventPhase {
    None = 0,
    Capturing = 1,
    AtTarget = 2,
    Bubbling = 3,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ScriptEventState {
    event_type: String,
    target: ListenerTarget,
    current_target: Option<ListenerTarget>,
    bubbles: bool,
    cancelable: bool,
    default_prevented: bool,
    propagation_stopped: bool,
    immediate_propagation_stopped: bool,
    phase: EventPhase,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScriptEventHandle(Rc<RefCell<ScriptEventState>>);

impl ScriptEventHandle {
    pub fn new(
        event_type: impl Into<String>,
        target: ListenerTarget,
        bubbles: bool,
        cancelable: bool,
    ) -> Self {
        Self(Rc::new(RefCell::new(ScriptEventState {
            event_type: event_type.into(),
            target,
            current_target: None,
            bubbles,
            cancelable,
            default_prevented: false,
            propagation_stopped: false,
            immediate_propagation_stopped: false,
            phase: EventPhase::None,
        })))
    }

    pub fn event_type(&self) -> String {
        self.0.borrow().event_type.clone()
    }

    pub fn target(&self) -> ListenerTarget {
        self.0.borrow().target
    }

    pub fn current_target(&self) -> Option<ListenerTarget> {
        self.0.borrow().current_target
    }

    pub fn set_current_target(&self, target: Option<ListenerTarget>) {
        self.0.borrow_mut().current_target = target;
    }

    pub fn bubbles(&self) -> bool {
        self.0.borrow().bubbles
    }

    pub fn cancelable(&self) -> bool {
        self.0.borrow().cancelable
    }

    pub fn default_prevented(&self) -> bool {
        self.0.borrow().default_prevented
    }

    pub fn propagation_stopped(&self) -> bool {
        self.0.borrow().propagation_stopped
    }

    pub fn immediate_propagation_stopped(&self) -> bool {
        self.0.borrow().immediate_propagation_stopped
    }

    pub fn event_phase(&self) -> EventPhase {
        self.0.borrow().phase
    }

    pub fn set_phase(&self, phase: EventPhase) {
        self.0.borrow_mut().phase = phase;
    }

    pub fn prevent_default(&self) {
        let mut state = self.0.borrow_mut();
        if state.cancelable {
            state.default_prevented = true;
        }
    }

    pub fn stop_propagation(&self) {
        self.0.borrow_mut().propagation_stopped = true;
    }

    pub fn stop_immediate_propagation(&self) {
        let mut state = self.0.borrow_mut();
        state.propagation_stopped = true;
        state.immediate_propagation_stopped = true;
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum ScriptValue {
    Undefined,
    Null,
    Boolean(bool),
    Number(f64),
    String(String),
    Element(ElementHandle),
    NodeList(Vec<ElementHandle>),
    Document,
    Window,
    Event(ScriptEventHandle),
    Function(ScriptFunction),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScriptFunction {
    pub params: Vec<String>,
    pub body_source: String,
}

impl ScriptFunction {
    pub fn new(params: Vec<String>, body_source: impl Into<String>) -> Self {
        Self {
            params,
            body_source: body_source.into(),
        }
    }
}

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

    fn document_get_element_by_id(&mut self, _id: &str) -> Result<Option<ElementHandle>> {
        Err(ScriptError::phase_not_ready("document.getElementById"))
    }

    fn document_query_selector(&mut self, _selector: &str) -> Result<Option<ElementHandle>> {
        Err(ScriptError::phase_not_ready("document.querySelector"))
    }

    fn document_query_selector_all(&mut self, _selector: &str) -> Result<Vec<ElementHandle>> {
        Err(ScriptError::phase_not_ready("document.querySelectorAll"))
    }

    fn element_text_content(&mut self, _element: ElementHandle) -> Result<String> {
        Err(ScriptError::phase_not_ready("element.textContent"))
    }

    fn element_set_text_content(&mut self, _element: ElementHandle, _value: &str) -> Result<()> {
        Err(ScriptError::phase_not_ready(
            "element.textContent assignment",
        ))
    }

    fn element_value(&mut self, _element: ElementHandle) -> Result<String> {
        Err(ScriptError::phase_not_ready("element.value"))
    }

    fn element_set_value(&mut self, _element: ElementHandle, _value: &str) -> Result<()> {
        Err(ScriptError::phase_not_ready("element.value assignment"))
    }

    fn element_checked(&mut self, _element: ElementHandle) -> Result<bool> {
        Err(ScriptError::phase_not_ready("element.checked"))
    }

    fn element_set_checked(&mut self, _element: ElementHandle, _checked: bool) -> Result<()> {
        Err(ScriptError::phase_not_ready("element.checked assignment"))
    }

    fn element_query_selector(
        &mut self,
        _element: ElementHandle,
        _selector: &str,
    ) -> Result<Option<ElementHandle>> {
        Err(ScriptError::phase_not_ready("element.querySelector"))
    }

    fn element_query_selector_all(
        &mut self,
        _element: ElementHandle,
        _selector: &str,
    ) -> Result<Vec<ElementHandle>> {
        Err(ScriptError::phase_not_ready("element.querySelectorAll"))
    }

    fn element_matches(&mut self, _element: ElementHandle, _selector: &str) -> Result<bool> {
        Err(ScriptError::phase_not_ready("element.matches"))
    }

    fn element_closest(
        &mut self,
        _element: ElementHandle,
        _selector: &str,
    ) -> Result<Option<ElementHandle>> {
        Err(ScriptError::phase_not_ready("element.closest"))
    }

    fn register_event_listener(
        &mut self,
        _target: ListenerTarget,
        _event_type: &str,
        _handler: ScriptFunction,
    ) -> Result<()> {
        self.register_event_listener_with_capture(_target, _event_type, false, _handler)
    }

    fn register_event_listener_with_capture(
        &mut self,
        _target: ListenerTarget,
        _event_type: &str,
        _capture: bool,
        _handler: ScriptFunction,
    ) -> Result<()> {
        Err(ScriptError::phase_not_ready("addEventListener"))
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
        host.on_eval(code, source_name)?;
        let program = self.parser.parse_program(code)?;
        self.evaluator.eval_program(&program, host)
    }

    pub fn eval_program_with_bindings<H: HostBindings>(
        &mut self,
        code: &str,
        source_name: &str,
        host: &mut H,
        initial_bindings: BTreeMap<String, ScriptValue>,
    ) -> Result<()> {
        host.on_eval(code, source_name)?;
        let program = self.parser.parse_program(code)?;
        self.evaluator
            .eval_program_with_bindings(&program, host, initial_bindings)
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

impl ScriptParser {
    pub(crate) fn parse_program(&self, code: &str) -> Result<syntax::Program> {
        parser::parse_program(code)
    }
}

impl Evaluator {
    pub(crate) fn eval_program<H: HostBindings>(
        &self,
        program: &syntax::Program,
        host: &mut H,
    ) -> Result<()> {
        evaluator::eval_program(program, host)
    }

    pub(crate) fn eval_program_with_bindings<H: HostBindings>(
        &self,
        program: &syntax::Program,
        host: &mut H,
        initial_bindings: BTreeMap<String, ScriptValue>,
    ) -> Result<()> {
        evaluator::eval_program_with_bindings(program, host, initial_bindings)
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
            .eval_program("const value = 'x';", "inline-script", &mut host)
            .expect("host callback should succeed");
        assert_eq!(
            host.evals,
            vec![(
                "inline-script".to_string(),
                "const value = 'x';".to_string(),
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

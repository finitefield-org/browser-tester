use browser_tester::Harness;
use proptest::collection::vec;
use proptest::prelude::*;
use proptest::test_runner::{FileFailurePersistence, TestCaseResult};

const RUNTIME_PROPTEST_REGRESSION_FILE: &str =
    "tests/proptest-regressions/runtime_property_fuzz_test.txt";
const DEFAULT_RUNTIME_PROPTEST_CASES: u32 = 128;

const RERENDERING_FORM_HTML: &str = r#"
<div id="mount"></div>
<script>
const mount = document.getElementById("mount");
const state = { name: "", checked: false, events: 0 };

function render() {
  mount.innerHTML = `
    <input id="name" value="${state.name}">
    <input id="flag" type="checkbox" ${state.checked ? "checked" : ""}>
    <button id="commit">commit</button>
    <p id="snapshot">${state.name}|${state.checked}|${state.events}</p>
  `;

  const nameInput = document.getElementById("name");
  const flagInput = document.getElementById("flag");
  const commitButton = document.getElementById("commit");

  nameInput.addEventListener("input", () => {
    state.name = document.getElementById("name").value;
    state.events += 1;
    render();
  });

  flagInput.addEventListener("input", () => {
    state.checked = document.getElementById("flag").checked;
    state.events += 1;
    render();
  });

  commitButton.addEventListener("click", () => {
    state.events += 1;
    render();
  });
}

render();
</script>
"#;

#[derive(Clone, Debug)]
enum UiAction {
    TypeText(String),
    SetChecked(bool),
    ClickCommit,
    FocusName,
    BlurName,
}

fn env_proptest_cases(var_name: &str, default_cases: u32) -> u32 {
    std::env::var(var_name)
        .ok()
        .and_then(|raw| raw.parse::<u32>().ok())
        .filter(|value| *value > 0)
        .unwrap_or(default_cases)
}

fn runtime_proptest_cases() -> u32 {
    std::env::var("BROWSER_TESTER_RUNTIME_PROPTEST_CASES")
        .ok()
        .and_then(|raw| raw.parse::<u32>().ok())
        .filter(|value| *value > 0)
        .unwrap_or_else(|| {
            env_proptest_cases(
                "BROWSER_TESTER_PROPTEST_CASES",
                DEFAULT_RUNTIME_PROPTEST_CASES,
            )
        })
}

fn text_input_strategy() -> BoxedStrategy<String> {
    vec(
        prop_oneof![
            Just('a'),
            Just('b'),
            Just('c'),
            Just('x'),
            Just('y'),
            Just('z'),
            Just('0'),
            Just('1'),
            Just('2'),
            Just('3'),
            Just(' '),
            Just('-'),
            Just('_'),
        ],
        0..=10,
    )
    .prop_map(|chars| chars.into_iter().collect())
    .boxed()
}

fn ui_action_strategy() -> BoxedStrategy<UiAction> {
    prop_oneof![
        5 => text_input_strategy().prop_map(UiAction::TypeText),
        3 => any::<bool>().prop_map(UiAction::SetChecked),
        2 => Just(UiAction::ClickCommit),
        1 => Just(UiAction::FocusName),
        1 => Just(UiAction::BlurName),
    ]
    .boxed()
}

fn ui_action_sequence_strategy() -> BoxedStrategy<Vec<UiAction>> {
    vec(ui_action_strategy(), 1..=24).boxed()
}

fn run_action(harness: &mut Harness, action: &UiAction) -> browser_tester::Result<()> {
    match action {
        UiAction::TypeText(value) => harness.type_text("#name", value),
        UiAction::SetChecked(value) => harness.set_checked("#flag", *value),
        UiAction::ClickCommit => harness.click("#commit"),
        UiAction::FocusName => harness.focus("#name"),
        UiAction::BlurName => harness.blur("#name"),
    }
}

fn assert_runtime_sequence_is_stable(actions: &[UiAction]) -> TestCaseResult {
    let mut harness = Harness::from_html(RERENDERING_FORM_HTML)
        .map_err(|err| proptest::test_runner::TestCaseError::fail(format!("{err:?}")))?;

    for (step, action) in actions.iter().enumerate() {
        let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            run_action(&mut harness, action)
        }));

        match outcome {
            Err(_) => {
                prop_assert!(
                    false,
                    "action panicked at step {step}: {action:?}, actions={actions:?}"
                );
            }
            Ok(Err(error)) => {
                prop_assert!(
                    false,
                    "action returned error at step {step}: {action:?}, error={error:?}, actions={actions:?}"
                );
            }
            Ok(Ok(())) => {}
        }

        prop_assert!(
            harness.assert_exists("#name").is_ok(),
            "name input missing after step {step}: {action:?}"
        );
        prop_assert!(
            harness.assert_exists("#flag").is_ok(),
            "flag checkbox missing after step {step}: {action:?}"
        );
        prop_assert!(
            harness.assert_exists("#commit").is_ok(),
            "commit button missing after step {step}: {action:?}"
        );
        prop_assert!(
            harness.assert_exists("#snapshot").is_ok(),
            "snapshot paragraph missing after step {step}: {action:?}"
        );
    }

    Ok(())
}

proptest! {
    #![proptest_config(ProptestConfig {
        cases: runtime_proptest_cases(),
        failure_persistence: Some(Box::new(
            FileFailurePersistence::Direct(RUNTIME_PROPTEST_REGRESSION_FILE),
        )),
        .. ProptestConfig::default()
    })]

    #[test]
    fn runtime_rerendering_form_actions_do_not_panic(actions in ui_action_sequence_strategy()) {
        assert_runtime_sequence_is_stable(&actions)?;
    }
}

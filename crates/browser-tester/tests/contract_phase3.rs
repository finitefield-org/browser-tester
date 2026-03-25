use browser_tester::Harness;

#[test]
fn type_text_updates_value_and_dispatches_input_listener() -> browser_tester::Result<()> {
    let mut harness = Harness::from_html(
        "<input id='name'><div id='out'></div><script>document.getElementById('name').addEventListener('input', () => { document.getElementById('out').textContent = document.getElementById('name').value; });</script>",
    )?;

    harness.type_text("#name", "Alice")?;
    harness.assert_value("#name", "Alice")?;
    harness.assert_text("#out", "Alice")?;
    Ok(())
}

#[test]
fn click_toggles_checkbox_and_dispatches_input_listener() -> browser_tester::Result<()> {
    let mut harness = Harness::from_html(
        "<input id='agree' type='checkbox'><div id='out'></div><script>document.getElementById('agree').addEventListener('input', () => { document.getElementById('out').textContent = String(document.getElementById('agree').checked); });</script>",
    )?;

    harness.click("#agree")?;
    harness.assert_checked("#agree", true)?;
    harness.assert_text("#out", "true")?;
    Ok(())
}

#[test]
fn click_submit_button_triggers_form_submit_default_action() -> browser_tester::Result<()> {
    let mut harness = Harness::from_html(
        "<form id='profile'><input id='name'><button id='submit' type='submit'>Save</button></form><div id='out'></div><script>document.getElementById('profile').addEventListener('submit', () => { document.getElementById('out').textContent = document.getElementById('name').value; });</script>",
    )?;

    harness.type_text("#name", "Alice")?;
    harness.click("#submit")?;
    harness.assert_text("#out", "Alice")?;
    Ok(())
}

#[test]
fn submit_dispatches_form_submit_listener_directly() -> browser_tester::Result<()> {
    let mut harness = Harness::from_html(
        "<form id='profile'><input id='name'><button id='submit' type='submit'>Save</button></form><div id='out'></div><script>document.getElementById('profile').addEventListener('submit', () => { document.getElementById('out').textContent = document.getElementById('name').value; });</script>",
    )?;

    harness.type_text("#name", "Bob")?;
    harness.submit("#profile")?;
    harness.assert_text("#out", "Bob")?;
    Ok(())
}

#[test]
fn dispatch_runs_target_listener_without_default_action() -> browser_tester::Result<()> {
    let mut harness = Harness::from_html(
        "<button id='run'></button><div id='out'></div><script>document.getElementById('run').addEventListener('custom', () => { document.getElementById('out').textContent = 'custom'; });</script>",
    )?;

    harness.dispatch("#run", "custom")?;
    harness.assert_text("#out", "custom")?;
    Ok(())
}

#[test]
fn type_text_on_non_form_control_reports_a_dom_error() -> browser_tester::Result<()> {
    let mut harness = Harness::from_html("<input id='out' type='checkbox'>")?;

    let error = harness
        .type_text("#out", "Alice")
        .expect_err("type_text should reject non-form controls");

    let message = error.to_string();
    assert!(message.contains("DOM error"));
    assert!(message.contains("text-like inputs and textareas"));
    Ok(())
}

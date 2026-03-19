use browser_tester_next::Harness;

#[test]
fn click_events_bubble_beyond_the_target_phase() -> browser_tester_next::Result<()> {
    let mut harness = Harness::from_html(
        "<div id='parent'><div id='child'></div></div><div id='out'></div><script>document.getElementById('child').addEventListener('click', () => { document.getElementById('out').textContent = 'target'; }); document.getElementById('parent').addEventListener('click', () => { document.getElementById('out').textContent += ':parent'; }); document.addEventListener('click', () => { document.getElementById('out').textContent += ':document'; }); window.addEventListener('click', () => { document.getElementById('out').textContent += ':window'; });</script>",
    )?;

    harness.click("#child")?;
    harness.assert_text("#out", "target:parent:document:window")?;
    Ok(())
}

#[test]
fn prevent_default_cancels_click_default_action() -> browser_tester_next::Result<()> {
    let mut harness = Harness::from_html(
        "<input id='agree' type='checkbox'><div id='out'></div><script>document.getElementById('agree').addEventListener('click', (event) => { event.preventDefault(); }); document.getElementById('agree').addEventListener('change', () => { document.getElementById('out').textContent = String(document.getElementById('agree').checked); });</script>",
    )?;

    harness.click("#agree")?;
    harness.assert_checked("#agree", false)?;
    harness.assert_text("#out", "")?;
    Ok(())
}

#[test]
fn focus_and_blur_are_publicly_supported() -> browser_tester_next::Result<()> {
    let mut harness = Harness::from_html(
        "<input id='first'><input id='second'><div id='out'></div><script>document.getElementById('first').addEventListener('blur', () => { document.getElementById('second').textContent = 'after-blur'; }); document.getElementById('second').addEventListener('focus', () => { document.getElementById('out').textContent = document.getElementById('second').textContent; });</script>",
    )?;

    harness.focus("#first")?;
    harness.focus("#second")?;
    harness.assert_text("#out", "after-blur")?;
    Ok(())
}

#[test]
fn set_select_value_updates_selection_and_fires_change() -> browser_tester_next::Result<()> {
    let mut harness = Harness::from_html(
        "<select id='mode'><option value='a'>A</option><option value='b'>B</option></select><div id='out'></div><script>document.getElementById('mode').addEventListener('change', () => { document.getElementById('out').textContent = document.getElementById('mode').value; });</script>",
    )?;

    harness.set_select_value("#mode", "b")?;
    harness.assert_value("#mode", "b")?;
    harness.assert_text("#out", "b")?;
    Ok(())
}

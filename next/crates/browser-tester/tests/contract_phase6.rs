use browser_tester_next::Harness;

#[test]
fn class_selectors_and_compound_selectors_work_end_to_end() -> browser_tester_next::Result<()> {
    let mut harness = Harness::from_html(
        "<main id='root'><button id='save' class='primary action'>Save</button><div id='out'></div><script>document.getElementById('save').addEventListener('click', () => { document.getElementById('out').textContent = 'clicked'; });</script></main>",
    )?;

    harness.assert_exists(".primary")?;
    harness.assert_exists("button.primary")?;
    harness.assert_exists("#save.primary")?;

    harness.click("button.primary")?;
    harness.assert_text("#out", "clicked")?;
    Ok(())
}

#[test]
fn descendant_combinators_work_end_to_end() -> browser_tester_next::Result<()> {
    let mut harness = Harness::from_html(
        "<main><section><button id='save' class='primary action'>Save</button></section><div id='out'></div><script>document.getElementById('save').addEventListener('click', () => { document.getElementById('out').textContent = 'clicked'; });</script></main>",
    )?;

    harness.assert_exists("main .primary")?;
    harness.assert_exists("main section .primary")?;

    harness.click("main .primary")?;
    harness.assert_text("#out", "clicked")?;
    Ok(())
}

#[test]
fn child_combinators_work_end_to_end() -> browser_tester_next::Result<()> {
    let mut harness = Harness::from_html(
        "<main><section><button id='save' class='primary action'>Save</button></section><div id='out'></div><script>document.getElementById('save').addEventListener('click', () => { document.getElementById('out').textContent = 'clicked'; });</script></main>",
    )?;

    harness.assert_exists("main > section > .primary")?;
    harness.assert_exists("main > section > button.primary")?;

    harness.click("main > section > button.primary")?;
    harness.assert_text("#out", "clicked")?;
    Ok(())
}

#[test]
fn adjacent_sibling_combinators_work_end_to_end() -> browser_tester_next::Result<()> {
    let mut harness = Harness::from_html(
        "<main><button id='first' class='primary'>First</button>text<button id='second' class='primary'>Second</button><div id='out'></div><script>document.getElementById('second').addEventListener('click', () => { document.getElementById('out').textContent = 'clicked'; });</script></main>",
    )?;

    harness.assert_exists("#first + .primary")?;
    harness.click("#first + .primary")?;
    harness.assert_text("#out", "clicked")?;
    Ok(())
}

#[test]
fn general_sibling_combinators_work_end_to_end() -> browser_tester_next::Result<()> {
    let mut harness = Harness::from_html(
        "<main><button id='first' class='primary'>First</button>text<button id='second' class='primary'>Second</button>text<button id='third' class='primary'>Third</button><div id='out'></div><script>document.getElementById('second').addEventListener('click', () => { document.getElementById('out').textContent = 'clicked'; });</script></main>",
    )?;

    harness.assert_exists("#first ~ .primary")?;
    harness.click("#first ~ .primary")?;
    harness.assert_text("#out", "clicked")?;
    Ok(())
}

#[test]
fn pseudo_class_selectors_still_fail_explicitly() -> browser_tester_next::Result<()> {
    let harness = Harness::from_html("<main><span class='primary'></span></main>")?;

    let error = harness
        .assert_exists("main:where([data-kind=primary x])")
        .expect_err("broader CSS parsing is not part of the selector slices");

    let message = error.to_string();
    assert!(message.contains("Selector error"));
    assert!(
        message.contains("supported forms are #id, .class, tag, tag.class, #id.class, [attr], [attr=value], [attr^=value], [attr$=value], [attr*=value], [attr~=value], [attr|=value], optional attribute selector flags like `[attr=value i]` and `[attr=value s]`, bounded logical pseudo-classes like `:not(.primary)`, `:is(.primary, .secondary)`, and `:where(.primary, .secondary)`, structural pseudo-classes like `:first-child`, `:last-child`, `:nth-child(2)`, `:nth-child(odd)`, `:nth-child(2n+1)`, and `:nth-last-child(2)`, state pseudo-classes like `:checked`, `:disabled`, and `:enabled`, descendant combinators like `A B`, adjacent sibling combinators like `A + B`, general sibling combinators like `A ~ B`, and child combinators like `A > B`")
    );
    Ok(())
}

#[test]
fn selector_hardening_keeps_public_actions_deterministic() -> browser_tester_next::Result<()> {
    let mut harness = Harness::from_html(
        "<main><form id='profile'><section><input id='name' class='field'></section><section><input id='agree' class='flag' type='checkbox'><select id='mode' class='mode'><option value='a'>A</option><option value='b'>B</option></select><button id='submit' type='submit' class='action'>Save</button></section></form><div id='out'></div><script>document.getElementById('profile').addEventListener('submit', () => { document.getElementById('out').textContent = document.getElementById('name').value + ':' + String(document.getElementById('agree').checked) + ':' + document.getElementById('mode').value; });</script></main>",
    )?;

    harness.type_text("main .field", "Ada")?;
    harness.set_checked("main > form > section > input.flag", true)?;
    harness.set_select_value("main > form > section > select.mode", "b")?;
    harness.submit("main > form")?;
    harness.assert_text("#out", "Ada:true:b")?;
    Ok(())
}

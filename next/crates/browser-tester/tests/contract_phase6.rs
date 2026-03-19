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
fn descendant_combinators_still_fail_explicitly() -> browser_tester_next::Result<()> {
    let harness = Harness::from_html("<main><span class='primary'></span></main>")?;

    let error = harness
        .assert_exists("main .primary")
        .expect_err("descendant combinators are not part of the first selector slice");

    let message = error.to_string();
    assert!(message.contains("Selector error"));
    assert!(
        message.contains("supported forms are #id, .class, tag, tag.class, #id.class, and [attr]")
    );
    Ok(())
}

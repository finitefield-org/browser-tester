use browser_tester_next::Harness;

#[test]
fn script_dom_query_selectors_work_end_to_end() -> browser_tester_next::Result<()> {
    let harness = Harness::from_html(
        "<main id='root' class='primary'>scope<section><div class='primary'>inside</div></section></main><div id='out'></div><script>const docMatch = document.querySelector('.primary'); const scopedMatch = document.getElementById('root').querySelector('.primary'); const missing = document.getElementById('root').querySelector('.missing'); document.getElementById('out').textContent = docMatch.textContent + ':' + scopedMatch.textContent + ':' + String(missing);</script>",
    )?;

    harness.assert_text("#out", "scopeinside:inside:null")?;
    Ok(())
}

#[test]
fn script_element_matches_works_end_to_end() -> browser_tester_next::Result<()> {
    let harness = Harness::from_html(
        "<main id='root' class='primary'><section><div id='child' class='child'></div></section></main><div id='out'></div><script>const root = document.getElementById('root'); const child = document.getElementById('child'); document.getElementById('out').textContent = String(root.matches('.primary')) + ':' + String(root.matches('.child')) + ':' + String(child.matches('.child'));</script>",
    )?;

    harness.assert_text("#out", "true:false:true")?;
    Ok(())
}

#[test]
fn script_element_closest_works_end_to_end() -> browser_tester_next::Result<()> {
    let harness = Harness::from_html(
        "<main id='root' class='primary'>ROOT<section id='section'>SECTION<div id='child' class='child'>CHILD</div></section></main><div id='out'></div><script>const root = document.getElementById('root'); const child = document.getElementById('child'); document.getElementById('out').textContent = root.closest('.primary').textContent + ':' + child.closest('.child').textContent + ':' + child.closest('#section').textContent + ':' + String(child.closest('.missing'));</script>",
    )?;

    harness.assert_text("#out", "ROOTSECTIONCHILD:CHILD:SECTIONCHILD:null")?;
    Ok(())
}

#[test]
fn unsupported_script_selector_methods_fail_explicitly() {
    let error = Harness::from_html(
        "<main id='out'></main><script>document.querySelectorAll('#out').textContent = 'Hello';</script>",
    )
    .expect_err("unsupported selector methods should fail");

    let message = error.to_string();
    assert!(message.contains("Script error"));
    assert!(message.contains("unsupported Document method: querySelectorAll"));
}

#[test]
fn unsupported_element_matches_selector_fails_explicitly() {
    let error = Harness::from_html(
        "<main id='root' class='primary'></main><script>document.getElementById('root').matches('main ~ .primary');</script>",
    )
    .expect_err("unsupported selector syntax should fail");

    let message = error.to_string();
    assert!(message.contains("Script error"));
    assert!(message.contains("supported forms are #id, .class, tag, tag.class, #id.class, [attr], descendant combinators like `A B`, adjacent sibling combinators like `A + B`, and child combinators like `A > B`"));
}

#[test]
fn unsupported_element_closest_selector_fails_explicitly() {
    let error = Harness::from_html(
        "<main id='root' class='primary'></main><script>document.getElementById('root').closest('main ~ .primary');</script>",
    )
    .expect_err("unsupported selector syntax should fail");

    let message = error.to_string();
    assert!(message.contains("Script error"));
    assert!(message.contains("supported forms are #id, .class, tag, tag.class, #id.class, [attr], descendant combinators like `A B`, adjacent sibling combinators like `A + B`, and child combinators like `A > B`"));
}

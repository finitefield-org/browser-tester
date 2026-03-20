use browser_tester_next::Harness;

#[test]
fn from_html_builds_dom_and_supports_phase_one_selectors() -> browser_tester_next::Result<()> {
    let harness = Harness::from_html(
        "<main id='app'><span data-state='Ready' data-tags='Ready NOW' lang='EN-US' data-label='Hello World'>Hello</span><input disabled></main>",
    )?;

    harness.assert_exists("#app")?;
    harness.assert_exists("main")?;
    harness.assert_exists("[data-state]")?;
    harness.assert_exists("[data-state=ready i]")?;
    harness.assert_exists("[data-state=Ready s]")?;
    harness.assert_exists("[data-state^=rea i]")?;
    harness.assert_exists("[data-tags~=ready i]")?;
    harness.assert_exists("[data-label='hello world' i]")?;
    harness.assert_exists("[data-label$=world i]")?;
    harness.assert_exists("[data-label*='LO WO' i]")?;
    harness.assert_exists("[lang|=en i]")?;
    harness.assert_exists("[lang|=EN s]")?;
    harness.assert_exists("[disabled='']")?;

    assert_eq!(
        harness.debug().dump_dom(),
        "#document\n  <main id=\"app\">\n    <span data-label=\"Hello World\" data-state=\"Ready\" data-tags=\"Ready NOW\" lang=\"EN-US\">\n      \"Hello\"\n    </span>\n    <input disabled />\n  </main>"
    );
    Ok(())
}

#[test]
fn selector_lists_work_with_public_assert_exists() -> browser_tester_next::Result<()> {
    let harness = Harness::from_html(
        "<main id='root' class='primary'>root</main><div class='primary'>inside</div>",
    )?;

    harness.assert_exists("main, .primary")?;
    harness.assert_exists(".primary, main")?;
    Ok(())
}

#[test]
fn simple_pseudo_classes_work_with_public_assert_exists() -> browser_tester_next::Result<()> {
    let harness = Harness::from_html(
        "<main>lead<!-- gap --><button id='first' class='primary'>First</button><button id='disabled' class='primary' disabled>Disabled</button><button id='enabled' class='primary'>Enabled</button><input id='agree' type='checkbox' checked><select id='mode'><option value='a'>A</option><option id='selected' value='b' selected>B</option></select></main>",
    )?;

    harness.assert_exists("#first:first-child")?;
    harness.assert_exists("button:nth-child(2)")?;
    harness.assert_exists("button:nth-child(3)")?;
    harness.assert_exists("button:nth-child(odd)")?;
    harness.assert_exists("button:nth-child(even)")?;
    harness.assert_exists("button:nth-child(2n+1)")?;
    harness.assert_exists("button:nth-child(-n+2)")?;
    harness.assert_exists("button:nth-last-child(5)")?;
    harness.assert_exists("button:nth-last-child(4)")?;
    harness.assert_exists("button:nth-last-child(odd)")?;
    harness.assert_exists("button:nth-last-child(even)")?;
    harness.assert_exists("button:nth-last-child(2n+1)")?;
    harness.assert_exists("button:disabled")?;
    harness.assert_exists("button:enabled")?;
    harness.assert_exists("input:checked")?;
    harness.assert_exists("option:checked")?;
    harness.assert_exists("select:last-child")?;
    Ok(())
}

#[test]
fn not_pseudo_class_works_with_public_assert_exists() -> browser_tester_next::Result<()> {
    let harness = Harness::from_html(
        "<main id='root' class='app' data-kind='APP READY' lang='EN-US'><button id='first' class='primary'>First</button><button id='disabled' class='primary' disabled>Disabled</button><button id='enabled' class='secondary'>Enabled</button></main>",
    )?;

    harness.assert_exists("main:not(.blocked)")?;
    harness.assert_exists("main:not(section .app, .blocked)")?;
    harness.assert_exists("main:not([data-kind~=blocked i], .blocked)")?;
    harness.assert_exists("button:not(:disabled)")?;
    harness.assert_exists("button:not(.secondary)")?;
    harness.assert_exists("button:not(:nth-child(even))")?;
    harness.assert_exists("button:not(main > .secondary, :disabled)")?;
    Ok(())
}

#[test]
fn is_pseudo_class_works_with_public_assert_exists() -> browser_tester_next::Result<()> {
    let harness = Harness::from_html(
        "<main id='root' class='app' data-kind='APP READY' lang='EN-US'><button id='first' class='primary'>First</button><button id='disabled' class='primary' disabled>Disabled</button><button id='enabled' class='secondary'>Enabled</button></main>",
    )?;

    harness.assert_exists("main:is(.app, .blocked)")?;
    harness.assert_exists("main:is([lang|=en i], .blocked)")?;
    harness.assert_exists("main:is([lang|=EN s], .blocked)")?;
    harness.assert_exists("button:is(:disabled, .secondary)")?;
    harness.assert_exists("button:is(main > .secondary, :disabled)")?;
    harness.assert_exists("button:is(.primary, .secondary):not(:disabled)")?;
    Ok(())
}

#[test]
fn where_pseudo_class_works_with_public_assert_exists() -> browser_tester_next::Result<()> {
    let harness = Harness::from_html(
        "<main id='root' class='app' data-kind='APP READY' lang='EN-US'><button id='first' class='primary'>First</button><button id='disabled' class='primary' disabled>Disabled</button><button id='enabled' class='secondary'>Enabled</button></main>",
    )?;

    harness.assert_exists("main:where(.app, .blocked)")?;
    harness.assert_exists("main:where([lang|=en i], .blocked)")?;
    harness.assert_exists("main:where([lang|=EN s], .blocked)")?;
    harness.assert_exists("button:where(:disabled, .secondary)")?;
    harness.assert_exists("button:where(main > .secondary, :disabled)")?;
    harness.assert_exists("button:where(.primary, .secondary):not(:disabled)")?;
    Ok(())
}

#[test]
fn assert_exists_reports_missing_nodes_with_dom_dump() -> browser_tester_next::Result<()> {
    let harness = Harness::from_html("<main id='app'></main>")?;

    let error = harness
        .assert_exists("#missing")
        .expect_err("missing nodes should fail");

    let message = error.to_string();
    assert!(message.contains("expected selector `#missing` to match at least one node"));
    assert!(message.contains("#document"));
    assert!(message.contains("<main id=\"app\" />"));
    Ok(())
}

#[test]
fn malformed_html_is_rejected_with_a_parse_error() {
    let error = Harness::from_html("<main><span></main>").expect_err("malformed HTML should fail");

    let message = error.to_string();
    assert!(message.contains("HTML parse error"));
    assert!(message.contains("mismatched closing tag"));
}

#[test]
fn unsupported_selector_syntax_is_reported_explicitly() -> browser_tester_next::Result<()> {
    let harness = Harness::from_html("<main><span class='app'></span></main>")?;
    let error = harness
        .assert_exists("main:where([data-kind=app x])")
        .expect_err("broader CSS parsing inside :where is not part of the selector slice");

    let message = error.to_string();
    assert!(message.contains("Selector error"));
    assert!(
        message.contains("supported forms are #id, .class, tag, tag.class, #id.class, [attr], [attr=value], [attr^=value], [attr$=value], [attr*=value], [attr~=value], [attr|=value], optional attribute selector flags like `[attr=value i]` and `[attr=value s]`, bounded logical pseudo-classes like `:not(.primary)`, `:is(.primary, .secondary)`, and `:where(.primary, .secondary)`, structural pseudo-classes like `:first-child`, `:last-child`, `:nth-child(2)`, `:nth-child(odd)`, `:nth-child(2n+1)`, and `:nth-last-child(2)`, state pseudo-classes like `:checked`, `:disabled`, and `:enabled`, descendant combinators like `A B`, adjacent sibling combinators like `A + B`, general sibling combinators like `A ~ B`, and child combinators like `A > B`")
    );
    Ok(())
}

#[test]
fn unsupported_not_argument_syntax_is_reported_explicitly() -> browser_tester_next::Result<()> {
    let harness = Harness::from_html("<main class='app'></main>")?;
    let error = harness
        .assert_exists("main:not([data-kind=app x])")
        .expect_err("broader CSS parsing inside :not is not part of the selector slice");

    let message = error.to_string();
    assert!(message.contains("Selector error"));
    assert!(
        message.contains("supported forms are #id, .class, tag, tag.class, #id.class, [attr], [attr=value], [attr^=value], [attr$=value], [attr*=value], [attr~=value], [attr|=value], optional attribute selector flags like `[attr=value i]` and `[attr=value s]`, bounded logical pseudo-classes like `:not(.primary)`, `:is(.primary, .secondary)`, and `:where(.primary, .secondary)`, structural pseudo-classes like `:first-child`, `:last-child`, `:nth-child(2)`, `:nth-child(odd)`, `:nth-child(2n+1)`, and `:nth-last-child(2)`, state pseudo-classes like `:checked`, `:disabled`, and `:enabled`, descendant combinators like `A B`, adjacent sibling combinators like `A + B`, general sibling combinators like `A ~ B`, and child combinators like `A > B`")
    );
    Ok(())
}

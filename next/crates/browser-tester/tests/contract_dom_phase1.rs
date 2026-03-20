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
fn selector_escapes_and_selector_lists_handle_literal_punctuation()
-> browser_tester_next::Result<()> {
    let harness = Harness::from_html(
        "<main id='root' class='app'><button id='foo,bar' class='alpha:beta'>First</button><button id='second' class='secondary'>Second</button></main>",
    )?;

    harness.assert_exists("#foo\\,bar")?;
    harness.assert_exists(".alpha\\:beta")?;
    harness.assert_exists("#foo\\,bar, .secondary")?;
    harness.assert_exists("main:is(#foo\\)bar, .app)")?;
    harness.assert_exists("button:where(#foo\\,bar, .secondary)")?;
    Ok(())
}

#[test]
fn selector_hex_escapes_work_with_public_assert_exists() -> browser_tester_next::Result<()> {
    let harness = Harness::from_html(
        "<main id='root' class='app'><button id='foo,bar' class='alpha:beta' data-label='foo]bar'>First</button><button id='second' class='secondary'>Second</button></main>",
    )?;

    harness.assert_exists("#foo\\2c bar")?;
    harness.assert_exists(".alpha\\3a beta")?;
    harness.assert_exists("[data-label=foo\\5d bar]")?;
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
fn root_and_empty_pseudo_classes_work_with_public_assert_exists() -> browser_tester_next::Result<()>
{
    let harness = Harness::from_html(
        "<main id='root'><section id='empty-comment'><!-- gap --></section><section id='empty'></section><section id='non-empty'>content</section></main>",
    )?;

    harness.assert_exists(":root")?;
    harness.assert_exists("main:root")?;
    harness.assert_exists("#empty-comment:empty")?;
    harness.assert_exists("#empty:empty")?;
    harness.assert_exists("#non-empty:not(:empty)")?;
    Ok(())
}

#[test]
fn only_child_and_only_of_type_pseudo_classes_work_with_public_assert_exists()
-> browser_tester_next::Result<()> {
    let harness = Harness::from_html(
        "<main id='root'><div id='single-child-parent'>lead<!-- gap --><section id='only-child'>child</section><!-- gap --></div><div id='type-parent'><span id='first-span'>one</span><em id='only-of-type'>type</em><span id='second-span'>two</span></div></main>",
    )?;

    harness.assert_exists("#only-child:only-child")?;
    harness.assert_exists("#type-parent > #only-of-type:only-of-type")?;
    harness.assert_exists("#first-span:not(:only-child)")?;
    harness.assert_exists("#first-span:not(:only-of-type)")?;
    Ok(())
}

#[test]
fn first_last_and_nth_of_type_pseudo_classes_work_with_public_assert_exists()
-> browser_tester_next::Result<()> {
    let harness = Harness::from_html(
        "<main id='root'><div id='type-parent'><span id='first-span'>one</span><em id='first-em'>first</em><span id='middle-span'>two</span><em id='last-em'>last</em><span id='last-span'>three</span></div></main>",
    )?;

    harness.assert_exists("#first-span:first-of-type")?;
    harness.assert_exists("#last-span:last-of-type")?;
    harness.assert_exists("#middle-span:nth-of-type(2)")?;
    harness.assert_exists("#middle-span:nth-last-of-type(2)")?;
    harness.assert_exists("#first-em:first-of-type")?;
    harness.assert_exists("#last-em:last-of-type")?;
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

#[test]
fn unsupported_hex_escape_selector_syntax_is_reported_explicitly() -> browser_tester_next::Result<()>
{
    let harness = Harness::from_html("<main id='foo,bar' class='app'></main>")?;
    let error = harness
        .assert_exists("#foo\\110000 bar")
        .expect_err("out-of-range hex escape should fail explicitly");

    let message = error.to_string();
    assert!(message.contains("Selector error"));
    assert!(message.contains("unsupported selector `#foo\\110000 bar`"));
    Ok(())
}

#[test]
fn unsupported_control_character_hex_escape_selector_syntax_is_reported_explicitly()
-> browser_tester_next::Result<()> {
    let harness = Harness::from_html("<main id='foo'></main>")?;
    let error = harness
        .assert_exists("#foo\\0 bar")
        .expect_err("control-character hex escape should fail explicitly");

    let message = error.to_string();
    assert!(message.contains("Selector error"));
    assert!(message.contains("unsupported selector `#foo\\0 bar`"));
    Ok(())
}

#[test]
fn unsupported_root_empty_selector_syntax_is_reported_explicitly() -> browser_tester_next::Result<()>
{
    let harness = Harness::from_html("<main id='root'><section id='empty'></section></main>")?;
    let error = harness
        .assert_exists("#empty:empty()")
        .expect_err("malformed :empty selector should fail explicitly");

    let message = error.to_string();
    assert!(message.contains("Selector error"));
    assert!(message.contains("unsupported selector `#empty:empty()`"));
    Ok(())
}

#[test]
fn unsupported_only_child_selector_syntax_is_reported_explicitly() -> browser_tester_next::Result<()>
{
    let harness = Harness::from_html("<main id='root'><section id='child'>child</section></main>")?;
    let error = harness
        .assert_exists("#child:only-child()")
        .expect_err("malformed :only-child selector should fail explicitly");

    let message = error.to_string();
    assert!(message.contains("Selector error"));
    assert!(message.contains("unsupported selector `#child:only-child()`"));
    Ok(())
}

#[test]
fn unsupported_first_of_type_selector_syntax_is_reported_explicitly()
-> browser_tester_next::Result<()> {
    let harness = Harness::from_html("<main id='root'><section id='child'>child</section></main>")?;
    let error = harness
        .assert_exists("#child:first-of-type()")
        .expect_err("malformed :first-of-type selector should fail explicitly");

    let message = error.to_string();
    assert!(message.contains("Selector error"));
    assert!(message.contains("unsupported selector `#child:first-of-type()`"));
    Ok(())
}

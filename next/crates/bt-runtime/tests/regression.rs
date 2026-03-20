use std::collections::BTreeMap;

use bt_runtime::{MockRegistry, Session, SessionConfig};

#[test]
fn reset_all_clears_every_mock_family() {
    let mut registry = MockRegistry::default();

    registry
        .fetch_mut()
        .respond_text("https://example.test/api/message", 200, "ok");
    registry
        .fetch_mut()
        .fail("https://example.test/api/error", "network disabled");
    registry
        .fetch_mut()
        .record_call("https://example.test/api/message");

    registry.dialogs_mut().push_confirm(true);
    registry.dialogs_mut().push_prompt(Some("Ada"));
    registry.dialogs_mut().record_alert("Notice");
    registry.dialogs_mut().record_confirm("Continue?");
    registry.dialogs_mut().record_prompt("Name?");

    registry.clipboard_mut().seed_text("seeded");
    registry.clipboard_mut().record_write("copied");

    registry
        .location_mut()
        .set_current("https://example.test/next");
    registry
        .location_mut()
        .record_navigation("https://example.test/next");

    registry
        .downloads_mut()
        .capture("report.csv", b"downloaded bytes".to_vec());

    registry
        .file_input_mut()
        .set_files("#upload", ["report.csv"]);

    registry.storage_mut().seed_local("token", "abc");
    registry.storage_mut().seed_session("session-token", "xyz");

    registry.reset_all();

    assert!(registry.fetch().responses().is_empty());
    assert!(registry.fetch().errors().is_empty());
    assert!(registry.fetch().calls().is_empty());
    assert!(registry.dialogs().confirm_queue().is_empty());
    assert!(registry.dialogs().prompt_queue().is_empty());
    assert!(registry.dialogs().alert_messages().is_empty());
    assert!(registry.dialogs().confirm_messages().is_empty());
    assert!(registry.dialogs().prompt_messages().is_empty());
    assert!(registry.clipboard().seeded_text().is_none());
    assert!(registry.clipboard().writes().is_empty());
    assert!(registry.location().current_url().is_none());
    assert!(registry.location().navigations().is_empty());
    assert!(registry.downloads().artifacts().is_empty());
    assert!(registry.file_input().selections().is_empty());
    assert!(registry.storage().local().is_empty());
    assert!(registry.storage().session().is_empty());
}

#[test]
fn session_rejects_unsupported_selector_syntax_in_closest_explicitly() {
    let error = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root' class='primary'></main><script>document.getElementById('root').closest('main:where([data-kind=primary x])');</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect_err("broader CSS parsing inside :where should fail explicitly");

    assert!(error.to_string().contains("Script error"));
    assert!(error.to_string().contains("supported forms are #id, .class, tag, tag.class, #id.class, [attr], [attr=value], [attr^=value], [attr$=value], [attr*=value], [attr~=value], [attr|=value], optional attribute selector flags like `[attr=value i]` and `[attr=value s]`, bounded logical pseudo-classes like `:not(.primary)`, `:is(.primary, .secondary)`, and `:where(.primary, .secondary)`, structural pseudo-classes like `:first-child`, `:last-child`, `:nth-child(2)`, `:nth-child(odd)`, `:nth-child(2n+1)`, and `:nth-last-child(2)`, state pseudo-classes like `:checked`, `:disabled`, and `:enabled`, descendant combinators like `A B`, adjacent sibling combinators like `A + B`, general sibling combinators like `A ~ B`, and child combinators like `A > B`"));
}

#[test]
fn session_resolves_selectors_with_quoted_commas_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root' class='app'><button id='first' data-label='A,B'>First</button><button id='second' class='secondary'>Second</button></main><div id='out'></div><script>const list = document.querySelectorAll(\"button[data-label='A,B'], .secondary\"); const isMatch = document.getElementById('root').matches(\"main:is([data-label='A,B'], .app)\"); const notMatch = document.getElementById('second').matches(\"button:not([data-label='A,B'], .blocked)\"); const whereMatch = document.getElementById('root').closest(\"main:where([data-label='A,B'], .app)\"); document.getElementById('out').textContent = String(list.length) + ':' + list.item(0).textContent + ':' + list.item(1).textContent + ':' + String(isMatch) + ':' + String(notMatch) + ':' + whereMatch.textContent;</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("quoted commas should remain supported inside bounded selector grammar");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "2:First:Second:true:true:FirstSecond"
    );
}

#[test]
fn session_resolves_selectors_with_escaped_punctuation_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root' class='app'><button id='foo,bar' class='alpha:beta'>First</button><button id='second' class='secondary'>Second</button><div id='out'></div></main><script>const escapedId = document.querySelector('#foo\\\\,bar'); const escapedClass = document.querySelector('.alpha\\\\:beta'); const list = document.querySelectorAll('#foo\\\\,bar, .secondary'); const isMatch = document.getElementById('root').matches('main:is(#foo\\\\)bar, .app)'); const whereMatch = document.getElementById('second').closest('button:where(#foo\\\\,bar, .secondary)'); document.getElementById('out').textContent = escapedId.textContent + ':' + escapedClass.textContent + ':' + String(list.length) + ':' + list.item(0).textContent + ':' + list.item(1).textContent + ':' + String(isMatch) + ':' + whereMatch.textContent;</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("escaped punctuation should remain supported inside bounded selector grammar");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "First:First:2:First:Second:true:Second"
    );
}

#[test]
fn session_resolves_selectors_with_hex_escapes_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root' class='app'><button id='foo,bar' class='alpha:beta' data-label='foo]bar'>First</button><button id='foo)bar' class='secondary'>Second</button><div id='out'></div></main><script>const escapedId = document.querySelector('#foo\\\\2c bar'); const escapedClass = document.querySelector('.alpha\\\\3a beta'); const escapedAttr = document.querySelector('[data-label=foo\\\\5d bar]'); const whereMatch = document.getElementById('foo)bar').closest('button:where(#foo\\\\29 bar, .secondary)'); document.getElementById('out').textContent = escapedId.textContent + ':' + escapedClass.textContent + ':' + escapedAttr.textContent + ':' + whereMatch.textContent;</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("hex escapes should resolve through Session-backed selector paths");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "First:First:First:Second"
    );
}

#[test]
fn session_rejects_out_of_range_hex_escape_selectors_explicitly() {
    let error = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root'></main><script>document.querySelector('#foo\\\\110000 bar');</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect_err("out-of-range hex escapes should fail explicitly");

    assert!(error.to_string().contains("Script error"));
    assert!(error.to_string().contains("supported forms are #id, .class, tag, tag.class, #id.class, [attr], [attr=value], [attr^=value], [attr$=value], [attr*=value], [attr~=value], [attr|=value], optional attribute selector flags like `[attr=value i]` and `[attr=value s]`, bounded logical pseudo-classes like `:not(.primary)`, `:is(.primary, .secondary)`, and `:where(.primary, .secondary)`, structural pseudo-classes like `:first-child`, `:last-child`, `:nth-child(2)`, `:nth-child(odd)`, `:nth-child(2n+1)`, and `:nth-last-child(2)`, state pseudo-classes like `:checked`, `:disabled`, and `:enabled`, descendant combinators like `A B`, adjacent sibling combinators like `A + B`, general sibling combinators like `A ~ B`, and child combinators like `A > B`"));
}

#[test]
fn session_rejects_control_character_hex_escape_selectors_explicitly() {
    let error = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='foo'></main><script>document.querySelector('#foo\\\\0 bar');</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect_err("control-character hex escapes should fail explicitly");

    assert!(error.to_string().contains("Script error"));
    assert!(
        error
            .to_string()
            .contains("unsupported selector `#foo\\0 bar`")
    );
}

#[test]
fn session_resolves_html_collection_named_item_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root'><span name='alpha'>First</span><span id='second'>Second</span></main><div id='out'></div><script>const children = document.getElementById('root').children; const alpha = children.namedItem('alpha'); document.getElementById('root').textContent = 'gone'; document.getElementById('out').textContent = alpha.textContent + ':' + String(children.namedItem('alpha'));</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("namedItem should remain available");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(session.dom().text_content_for_node(out_id), "First:null");
}

#[test]
fn session_resolves_get_elements_by_tag_name_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root'><span name='alpha'>First</span><span id='second'>Second</span></main><div id='out'></div><script>const all = document.getElementsByTagName('span'); const scoped = document.getElementById('root').getElementsByTagName('span'); const alpha = all.namedItem('alpha'); const before = all.length; document.getElementById('root').textContent = 'gone'; document.getElementById('out').textContent = String(before) + ':' + String(all.length) + ':' + String(scoped.length) + ':' + alpha.textContent + ':' + String(all.namedItem('alpha'));</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("getElementsByTagName should remain available");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "2:0:0:First:null"
    );
}

#[test]
fn session_resolves_get_elements_by_class_name_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root' class='alpha'><span name='alpha' class='alpha'>First</span><span id='second' class='alpha'>Second</span></main><div id='out'></div><script>const all = document.getElementsByClassName('alpha'); const scoped = document.getElementById('root').getElementsByClassName('alpha'); const named = all.namedItem('alpha'); const root = all.item(0); const before = all.length; const beforeScoped = scoped.length; document.getElementById('root').textContent = 'gone'; document.getElementById('out').textContent = String(before) + ':' + String(all.length) + ':' + String(beforeScoped) + ':' + String(scoped.length) + ':' + named.textContent + ':' + String(scoped.namedItem('alpha')) + ':' + root.textContent;</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("getElementsByClassName should remain available");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "3:1:2:0:First:null:gone"
    );
}

#[test]
fn session_resolves_get_elements_by_tag_name_ns_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<div id='root'><svg id='icon'><rect id='rect'></rect><circle id='dot'></circle></svg><math id='formula'><mi id='symbol'>x</mi></math><span id='label'>Label</span></div><div id='out'></div><script>const svgAll = document.getElementsByTagNameNS('http://www.w3.org/2000/svg', '*'); const svgRect = document.getElementById('icon').getElementsByTagNameNS('http://www.w3.org/2000/svg', 'rect'); const dot = svgAll.namedItem('dot'); document.getElementById('root').textContent = 'gone'; document.getElementById('out').textContent = String(svgAll.length) + ':' + String(svgRect.length) + ':' + String(dot) + ':' + String(svgAll.namedItem('dot'));</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("getElementsByTagNameNS should remain available");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "0:1:[object Element]:null"
    );
}

#[test]
fn session_resolves_nth_child_with_non_element_siblings_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main>lead<!-- gap --><button id='first'>First</button><button id='second'>Second</button><div id='out'></div><script>const second = document.querySelector('button:nth-child(2)'); const first = document.getElementById('first'); document.getElementById('out').textContent = second.textContent + ':' + String(first.matches('button:nth-child(1)')) + ':' + String(second.matches('button:nth-child(2)'));</script></main>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("nth-child should ignore non-element siblings");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "Second:true:true"
    );
}

#[test]
fn session_resolves_nth_child_formulas_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main>lead<!-- gap --><button id='first'>First</button><button id='second'>Second</button><button id='third'>Third</button><div id='out'></div><script>const odd = document.querySelectorAll('button:nth-child(odd)'); const even = document.querySelector('button:nth-child(even)'); const formula = document.querySelectorAll('button:nth-child(2n+1)'); const limited = document.querySelectorAll('button:nth-child(-n+2)'); document.getElementById('out').textContent = String(odd.length) + ':' + even.textContent + ':' + String(formula.length) + ':' + String(limited.length);</script></main>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("nth-child formulas should remain available");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(session.dom().text_content_for_node(out_id), "2:Second:2:2");
}

#[test]
fn session_resolves_nth_last_child_formulas_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main>lead<!-- gap --><button id='first'>First</button><button id='second'>Second</button><button id='third'>Third</button></main><div id='out'></div><script>const second = document.querySelector('button:nth-last-child(2)'); const odd = document.querySelectorAll('button:nth-last-child(odd)'); const even = document.querySelectorAll('button:nth-last-child(even)'); const formula = document.querySelector('button:nth-last-child(2n+1)'); document.getElementById('out').textContent = second.textContent + ':' + String(odd.length) + ':' + String(even.length) + ':' + formula.textContent;</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("nth-last-child formulas should remain available");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "Second:2:1:First"
    );
}

#[test]
fn session_resolves_not_pseudo_class_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root' class='app' data-kind='APP READY'><button id='first' class='primary'>First</button><button id='disabled' class='primary' disabled>Disabled</button><button id='enabled' class='secondary'>Enabled</button><div id='out'></div><script>const enabled = document.querySelectorAll('button:not(:disabled)'); const second = document.getElementById('enabled'); const root = second.closest('main:not([data-kind~=blocked i], .blocked)'); const bounded = document.querySelectorAll('button:not(main > .secondary, :disabled)'); document.getElementById('out').textContent = String(enabled.length) + ':' + enabled.item(0).textContent + ':' + enabled.item(1).textContent + ':' + String(second.matches('button:not(.primary)')) + ':' + String(root.matches('main:not([data-kind~=blocked i], .blocked)')) + ':' + document.querySelector('button:not(:nth-child(even))').textContent + ':' + String(bounded.length) + ':' + bounded.item(0).textContent;</script></main>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect(":not pseudo-class should remain available");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "2:First:Enabled:true:true:First:1:First"
    );
}

#[test]
fn session_resolves_is_pseudo_class_and_nested_selector_lists_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root' class='app' data-kind='APP READY' lang='EN-US'><button id='first' class='primary'>First</button><button id='disabled' class='primary' disabled>Disabled</button><button id='enabled' class='secondary'>Enabled</button></main><div id='out'></div><script>const outer = document.querySelectorAll('main, button:is(.primary, .secondary)'); const filtered = document.querySelectorAll('button:is(:disabled, .secondary)'); const bounded = document.querySelectorAll('button:is(main > .secondary, :disabled)'); const second = document.getElementById('enabled'); const root = second.closest('main:is([lang|=en i], .blocked)'); document.getElementById('out').textContent = String(outer.length) + ':' + String(filtered.length) + ':' + String(bounded.length) + ':' + String(second.matches('button:is(.secondary, .blocked)')) + ':' + String(root.matches('main:is([lang|=en i], .blocked)'));</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect(":is pseudo-class should remain available");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "4:2:2:true:true"
    );
}

#[test]
fn session_resolves_where_pseudo_class_and_nested_selector_lists_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root' class='app' data-kind='APP READY' lang='EN-US'><button id='first' class='primary'>First</button><button id='disabled' class='primary' disabled>Disabled</button><button id='enabled' class='secondary'>Enabled</button></main><div id='out'></div><script>const outer = document.querySelectorAll('main, button:where(.primary, .secondary)'); const filtered = document.querySelectorAll('button:where(:disabled, .secondary)'); const bounded = document.querySelectorAll('button:where(main > .secondary, :disabled)'); const second = document.getElementById('enabled'); const root = second.closest('main:where([lang|=en i], .blocked)'); document.getElementById('out').textContent = String(outer.length) + ':' + String(filtered.length) + ':' + String(bounded.length) + ':' + String(second.matches('button:where(.secondary, .blocked)')) + ':' + String(root.matches('main:where([lang|=en i], .blocked)'));</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect(":where pseudo-class should remain available");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "4:2:2:true:true"
    );
}

#[test]
fn session_resolves_root_and_empty_pseudo_classes_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root'><div id='empty'></div><div id='comment-only'><!-- marker --></div><div id='with-text'>x</div><div id='out'></div></main><script>const root = document.querySelector(':root'); const empty = document.querySelector('#empty:empty'); const commentOnly = document.querySelector('#comment-only:empty'); const nonEmpty = document.querySelector('#with-text:empty'); const isRoot = document.getElementById('root').matches(':root'); const childIsRoot = document.getElementById('empty').matches(':root'); const closestRoot = document.getElementById('with-text').closest(':root'); document.getElementById('out').textContent = String(root.matches(':root')) + ':' + String(empty.matches(':empty')) + ':' + String(commentOnly.matches(':empty')) + ':' + String(nonEmpty) + ':' + String(isRoot) + ':' + String(childIsRoot) + ':' + String(closestRoot.matches(':root'));</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect(":root and :empty pseudo-classes should remain available");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "true:true:true:null:true:false:true"
    );
}

#[test]
fn session_resolves_only_child_and_only_of_type_pseudo_classes_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root'>lead<!-- gap --><div id='single-child-parent'>text<!-- marker --><section id='only-child'>child</section><!-- marker --></div><div id='type-parent'><span id='first-span'>one</span><em id='only-of-type'>type</em><span id='second-span'>two</span></div><div id='out'></div><script>const onlyChild = document.querySelector('#only-child:only-child'); const onlyOfType = document.querySelector('#only-of-type:only-of-type'); const onlyChildMatches = document.querySelectorAll('#single-child-parent > :only-child'); const onlyOfTypeMatches = document.querySelectorAll('#type-parent > :only-of-type'); const firstSpan = document.getElementById('first-span'); const parent = onlyChild.closest('#single-child-parent'); document.getElementById('out').textContent = onlyChild.textContent + ':' + onlyOfType.textContent + ':' + String(onlyChildMatches.length) + ':' + String(onlyOfTypeMatches.length) + ':' + String(firstSpan.matches('#first-span:not(:only-child)')) + ':' + String(firstSpan.matches('#first-span:not(:only-of-type)')) + ':' + String(parent.matches('#single-child-parent'));</script></main>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect(":only-child and :only-of-type pseudo-classes should remain available");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "child:type:1:1:true:true:true"
    );
}

#[test]
fn session_resolves_first_last_and_nth_of_type_pseudo_classes_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root'><div id='type-parent'><span id='first-span'>one</span><em id='first-em'>first</em><span id='middle-span'>two</span><em id='last-em'>last</em><span id='last-span'>three</span></div><div id='out'></div><script>const firstSpan = document.querySelector('#first-span:first-of-type'); const lastSpan = document.querySelector('#last-span:last-of-type'); const middleSpan = document.querySelector('#middle-span:nth-of-type(2)'); const middleFromEnd = document.querySelector('#middle-span:nth-last-of-type(2)'); const firstEm = document.querySelector('#first-em:first-of-type'); const lastEm = document.querySelector('#last-em:last-of-type'); document.getElementById('out').textContent = String(firstSpan.matches('#first-span:first-of-type')) + ':' + String(lastSpan.matches('#last-span:last-of-type')) + ':' + String(middleSpan.matches('#middle-span:nth-of-type(2)')) + ':' + String(middleFromEnd.matches('#middle-span:nth-last-of-type(2)')) + ':' + String(firstEm.matches('#first-em:first-of-type')) + ':' + String(lastEm.matches('#last-em:last-of-type'));</script></main>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect(":first-of-type and :nth-of-type pseudo-classes should remain available");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "true:true:true:true:true:true"
    );
}

#[test]
fn session_rejects_unsupported_empty_pseudo_arguments_explicitly() {
    let error = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root'></main><script>document.querySelector('main:empty(1)');</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect_err("unsupported :empty arguments should fail explicitly");

    assert!(error.to_string().contains("Script error"));
    assert!(
        error
            .to_string()
            .contains("unsupported selector `main:empty(1)`")
    );
}

#[test]
fn session_rejects_unsupported_first_of_type_selector_syntax_explicitly() {
    let error = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root'><section id='child'>child</section></main><script>document.querySelector('#child:first-of-type()');</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect_err("malformed :first-of-type selector should fail explicitly");

    assert!(error.to_string().contains("Script error"));
    assert!(
        error
            .to_string()
            .contains("unsupported selector `#child:first-of-type()`")
    );
}

#[test]
fn session_rejects_unsupported_only_child_selector_syntax_explicitly() {
    let error = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root'><section id='child'>child</section></main><script>document.querySelector('#child:only-child()');</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect_err("malformed :only-child selector should fail explicitly");

    assert!(error.to_string().contains("Script error"));
    assert!(
        error
            .to_string()
            .contains("unsupported selector `#child:only-child()`")
    );
}

#[test]
fn session_rejects_unsupported_not_argument_syntax_explicitly() {
    let error = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root' class='primary'></main><script>document.getElementById('root').matches('main:not([data-kind=primary x])');</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect_err("broader CSS parsing inside :not should fail explicitly");

    assert!(error.to_string().contains("Script error"));
    assert!(error.to_string().contains("supported forms are #id, .class, tag, tag.class, #id.class, [attr], [attr=value], [attr^=value], [attr$=value], [attr*=value], [attr~=value], [attr|=value], optional attribute selector flags like `[attr=value i]` and `[attr=value s]`, bounded logical pseudo-classes like `:not(.primary)`, `:is(.primary, .secondary)`, and `:where(.primary, .secondary)`, structural pseudo-classes like `:first-child`, `:last-child`, `:nth-child(2)`, `:nth-child(odd)`, `:nth-child(2n+1)`, and `:nth-last-child(2)`, state pseudo-classes like `:checked`, `:disabled`, and `:enabled`, descendant combinators like `A B`, adjacent sibling combinators like `A + B`, general sibling combinators like `A ~ B`, and child combinators like `A > B`"));
}

#[test]
fn session_rejects_document_links_on_elements_explicitly() {
    let error = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<div id='wrapper'><div id='not-doc'></div></div><script>document.getElementById('not-doc').links.length;</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect_err("non-document links access should fail explicitly");

    assert!(error.to_string().contains("Script error"));
    assert!(error.to_string().contains("unsupported member access"));
    assert!(error.to_string().contains("`links`"));
    assert!(error.to_string().contains("element value"));
}

#[test]
fn session_rejects_document_all_on_elements_explicitly() {
    let error = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<div id='wrapper'><div id='not-doc'></div></div><script>document.getElementById('not-doc').all.length;</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect_err("non-document all access should fail explicitly");

    assert!(error.to_string().contains("Script error"));
    assert!(error.to_string().contains("unsupported member access"));
    assert!(error.to_string().contains("`all`"));
    assert!(error.to_string().contains("element value"));
}

#[test]
fn session_rejects_get_elements_by_name_on_elements_explicitly() {
    let error = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root'><span name='alpha'>First</span></main><script>document.getElementById('root').getElementsByName('alpha');</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect_err("element.getElementsByName should fail explicitly");

    assert!(error.to_string().contains("Script error"));
    assert!(
        error
            .to_string()
            .contains("unsupported Element method: getElementsByName")
    );
}

#[test]
fn session_rejects_form_elements_on_non_form_elements_explicitly() {
    let error = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<div id='wrapper'><div id='not-form'></div></div><script>document.getElementById('wrapper').elements.length;</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect_err("non-form elements should fail explicitly");

    assert!(error.to_string().contains("Script error"));
    assert!(error.to_string().contains("node is not a form element"));
}

#[test]
fn session_rejects_select_options_on_non_select_elements_explicitly() {
    let error = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<div id='wrapper'><div id='not-select'></div></div><script>document.getElementById('not-select').options.length;</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect_err("non-select elements should fail explicitly");

    assert!(error.to_string().contains("Script error"));
    assert!(error.to_string().contains("node is not a select element"));
}

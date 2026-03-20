use bt_dom::{
    DomStore, HTML_NAMESPACE_URI, MATHML_NAMESPACE_URI, NodeId, NodeKind, SVG_NAMESPACE_URI,
};

#[test]
fn phase_zero_store_exposes_document_root() {
    let store = DomStore::new_empty();

    assert_eq!(store.document_id(), NodeId::new(0, 0));
    assert_eq!(store.node_count(), 1);
    assert_eq!(store.nodes()[0].id, NodeId::new(0, 0));
    assert_eq!(store.nodes()[0].kind, NodeKind::Document);
    assert!(store.indexes().id_index.is_empty());
    assert!(store.side_tables().form_controls.is_empty());
    assert!(store.side_tables().file_inputs.is_empty());
    assert_eq!(store.document_state().title, "");
}

#[test]
fn phase_one_html_tree_building_and_selectors_work() {
    let mut store = DomStore::new_empty();
    store
        .bootstrap_html(
            "<main id='app'><span data-state='Ready' data-tags='Ready NOW' lang='EN-US' data-label='Hello World'>Hello</span><input disabled></main>",
        )
        .expect("HTML should parse");

    assert_eq!(
        store.source_html(),
        Some(
            "<main id='app'><span data-state='Ready' data-tags='Ready NOW' lang='EN-US' data-label='Hello World'>Hello</span><input disabled></main>"
        )
    );
    assert_eq!(store.node_count(), 5);
    assert_eq!(store.select("#app").unwrap(), vec![NodeId::new(1, 0)]);
    assert_eq!(store.select("main").unwrap(), vec![NodeId::new(1, 0)]);
    assert_eq!(
        store.select("[data-state]").unwrap(),
        vec![NodeId::new(2, 0)]
    );
    assert_eq!(
        store.select("[data-state=ready i]").unwrap(),
        vec![NodeId::new(2, 0)]
    );
    assert_eq!(
        store.select("[data-state=Ready s]").unwrap(),
        vec![NodeId::new(2, 0)]
    );
    assert_eq!(
        store.select("[data-state^=rea i]").unwrap(),
        vec![NodeId::new(2, 0)]
    );
    assert_eq!(
        store.select("[data-tags~=ready i]").unwrap(),
        vec![NodeId::new(2, 0)]
    );
    assert_eq!(
        store.select("[data-label='hello world' i]").unwrap(),
        vec![NodeId::new(2, 0)]
    );
    assert_eq!(
        store.select("[data-label$=world i]").unwrap(),
        vec![NodeId::new(2, 0)]
    );
    assert_eq!(
        store.select("[data-label*='LO WO' i]").unwrap(),
        vec![NodeId::new(2, 0)]
    );
    assert_eq!(
        store.select("[lang|=en i]").unwrap(),
        vec![NodeId::new(2, 0)]
    );
    assert_eq!(
        store.select("[lang|=EN s]").unwrap(),
        vec![NodeId::new(2, 0)]
    );
    assert_eq!(
        store.select("[disabled='']").unwrap(),
        vec![NodeId::new(4, 0)]
    );
    assert_eq!(
        store.dump_dom(),
        "#document\n  <main id=\"app\">\n    <span data-label=\"Hello World\" data-state=\"Ready\" data-tags=\"Ready NOW\" lang=\"EN-US\">\n      \"Hello\"\n    </span>\n    <input disabled />\n  </main>"
    );
}

#[test]
fn malformed_html_is_reported_explicitly() {
    let mut store = DomStore::new_empty();
    let error = store
        .bootstrap_html("<main><span></main>")
        .expect_err("mismatched tags should fail");

    assert!(error.contains("mismatched closing tag"));
}

#[test]
fn simple_pseudo_classes_match_state_and_structure() {
    let mut store = DomStore::new_empty();
    store
        .bootstrap_html(
            "<main>lead<!-- gap --><button id='first' class='primary'>First</button><button id='disabled' class='primary' disabled>Disabled</button><button id='enabled' class='primary'>Enabled</button><input id='agree' type='checkbox' checked><select id='mode'><option value='a'>A</option><option id='selected' value='b' selected>B</option></select></main>",
        )
        .expect("HTML should parse");

    let first_id = store.select("#first").unwrap()[0];
    let disabled_id = store.select("#disabled").unwrap()[0];
    let enabled_id = store.select("#enabled").unwrap()[0];
    let agree_id = store.select("#agree").unwrap()[0];
    let selected_id = store.select("#selected").unwrap()[0];
    let mode_id = store.select("#mode").unwrap()[0];

    assert_eq!(store.select("#first:first-child").unwrap(), vec![first_id]);
    assert_eq!(
        store.select("button:nth-child(2)").unwrap(),
        vec![disabled_id]
    );
    assert_eq!(
        store.select("button:nth-child(3)").unwrap(),
        vec![enabled_id]
    );
    assert_eq!(
        store.select("button:nth-child(odd)").unwrap(),
        vec![first_id, enabled_id]
    );
    assert_eq!(
        store.select("button:nth-child(even)").unwrap(),
        vec![disabled_id]
    );
    assert_eq!(
        store.select("button:nth-child(2n+1)").unwrap(),
        vec![first_id, enabled_id]
    );
    assert_eq!(
        store.select("button:nth-child(-n+2)").unwrap(),
        vec![first_id, disabled_id]
    );
    assert_eq!(
        store.select("button:nth-child( 2n + 1 )").unwrap(),
        vec![first_id, enabled_id]
    );
    assert_eq!(
        store.select("button:nth-last-child(5)").unwrap(),
        vec![first_id]
    );
    assert_eq!(
        store.select("button:nth-last-child(4)").unwrap(),
        vec![disabled_id]
    );
    assert_eq!(
        store.select("button:nth-last-child(odd)").unwrap(),
        vec![first_id, enabled_id]
    );
    assert_eq!(
        store.select("button:nth-last-child(even)").unwrap(),
        vec![disabled_id]
    );
    assert_eq!(
        store.select("button:nth-last-child(2n+1)").unwrap(),
        vec![first_id, enabled_id]
    );
    assert_eq!(store.select("button:disabled").unwrap(), vec![disabled_id]);
    assert_eq!(
        store.select("button:enabled").unwrap(),
        vec![first_id, enabled_id]
    );
    assert_eq!(store.select("input:checked").unwrap(), vec![agree_id]);
    assert_eq!(store.select("option:checked").unwrap(), vec![selected_id]);
    assert_eq!(store.select("select:last-child").unwrap(), vec![mode_id]);
}

#[test]
fn root_and_empty_pseudo_classes_match_expected_nodes() {
    let mut store = DomStore::new_empty();
    store
        .bootstrap_html(
            "<main id='root'><section id='empty'></section><div id='comment-only'><!-- marker --></div><p id='with-text'>text</p><article id='with-child'><span id='nested'>nested</span></article></main>",
        )
        .expect("HTML should parse");

    let root_id = store.select("#root").unwrap()[0];
    let empty_id = store.select("#empty").unwrap()[0];
    let comment_only_id = store.select("#comment-only").unwrap()[0];

    assert_eq!(store.select(":root").unwrap(), vec![root_id]);
    assert_eq!(
        store.select(":empty").unwrap(),
        vec![empty_id, comment_only_id]
    );

    // Failure-path regressions for structural pseudo-classes.
    assert!(store.select("#empty:root").unwrap().is_empty());
    assert!(store.select("#with-text:empty").unwrap().is_empty());
    assert!(store.select("#with-child:empty").unwrap().is_empty());
}

#[test]
fn only_child_and_only_of_type_pseudo_classes_match_expected_nodes() {
    let mut store = DomStore::new_empty();
    store
        .bootstrap_html(
            "<main id='root'>lead<!-- gap --><div id='single-child-parent'>text<!-- marker --><section id='only-child'>child</section><!-- marker --></div><div id='type-parent'><span id='first-span'>one</span><em id='only-of-type'>type</em><span id='second-span'>two</span></div></main>",
        )
        .expect("HTML should parse");

    let root_id = store.select("#root").unwrap()[0];
    let only_child_id = store.select("#only-child").unwrap()[0];
    let only_of_type_id = store.select("#only-of-type").unwrap()[0];

    assert_eq!(store.select(":root:only-child").unwrap(), vec![root_id]);
    assert_eq!(
        store.select("#only-child:only-child").unwrap(),
        vec![only_child_id]
    );
    assert_eq!(
        store.select("#only-of-type:only-of-type").unwrap(),
        vec![only_of_type_id]
    );
    assert!(store.select("#first-span:only-child").unwrap().is_empty());
    assert!(store.select("#first-span:only-of-type").unwrap().is_empty());
}

#[test]
fn attribute_reflection_mutation_updates_indexes_and_form_controls() {
    let mut store = DomStore::new_empty();
    store
        .bootstrap_html(
            "<main id='root' class='alpha beta' name='root-name' data-flag><input id='agree' type='checkbox'><input id='name' type='text' value='Alice'><select id='mode'><option id='opt-a' value='a'>A</option><option id='opt-b' value='b' selected>B</option></select><button id='btn'>Go</button></main>",
        )
        .expect("HTML should parse");

    let root_id = store.select("#root").unwrap()[0];
    let agree_id = store.select("#agree").unwrap()[0];
    let name_id = store.select("#name").unwrap()[0];
    let mode_id = store.select("#mode").unwrap()[0];
    let opt_a_id = store.select("#opt-a").unwrap()[0];
    let btn_id = store.select("#btn").unwrap()[0];

    assert_eq!(
        store.get_attribute(root_id, "id").unwrap(),
        Some("root".to_string())
    );
    assert_eq!(
        store.get_attribute(root_id, "DATA-FLAG").unwrap(),
        Some(String::new())
    );
    assert!(store.has_attribute(root_id, "data-flag").unwrap());
    assert_eq!(store.get_attribute(root_id, "missing").unwrap(), None);
    assert!(!store.has_attribute(root_id, "missing").unwrap());

    store
        .set_attribute(root_id, "ID", "renamed")
        .expect("set id should succeed");
    assert_eq!(store.select("#renamed").unwrap(), vec![root_id]);
    assert!(store.select("#root").unwrap().is_empty());
    assert_eq!(
        store.get_attribute(root_id, "id").unwrap(),
        Some("renamed".to_string())
    );

    store
        .set_attribute(root_id, "class", "gamma delta")
        .expect("set class should succeed");
    assert_eq!(store.select(".gamma").unwrap(), vec![root_id]);
    assert!(store.select(".alpha").unwrap().is_empty());

    store
        .set_attribute(root_id, "name", "new-name")
        .expect("set name should succeed");
    assert!(
        store
            .indexes()
            .name_index
            .get("new-name")
            .is_some_and(|nodes| nodes.contains(&root_id))
    );
    assert!(store.indexes().name_index.get("root-name").is_none());

    store
        .set_attribute(name_id, "value", "Bob")
        .expect("set value should succeed");
    assert_eq!(store.value_for_node(name_id), "Bob");

    store
        .set_attribute(agree_id, "checked", "")
        .expect("set checked should succeed");
    assert_eq!(store.checked_for_node(agree_id), Some(true));
    assert_eq!(store.select("input:checked").unwrap(), vec![agree_id]);

    let toggled = store
        .toggle_attribute(agree_id, "checked", None)
        .expect("toggle checked should succeed");
    assert!(!toggled);
    assert_eq!(store.checked_for_node(agree_id), Some(false));
    assert!(store.select("input:checked").unwrap().is_empty());

    let forced = store
        .toggle_attribute(agree_id, "checked", Some(true))
        .expect("force checked should succeed");
    assert!(forced);
    assert_eq!(store.checked_for_node(agree_id), Some(true));

    store
        .set_attribute(opt_a_id, "selected", "")
        .expect("set selected should succeed");
    assert_eq!(store.value_for_node(mode_id), "a");

    let removed = store
        .remove_attribute(root_id, "class")
        .expect("remove class should succeed");
    assert!(removed);
    assert!(store.select(".gamma").unwrap().is_empty());

    assert!(!store.has_attribute(btn_id, "disabled").unwrap());
    let disabled = store
        .toggle_attribute(btn_id, "disabled", None)
        .expect("toggle disabled should succeed");
    assert!(disabled);
    assert!(store.has_attribute(btn_id, "disabled").unwrap());
    assert_eq!(store.select("[disabled]").unwrap(), vec![btn_id]);
}

#[test]
fn attribute_reflection_rejects_invalid_nodes_and_names() {
    let mut store = DomStore::new_empty();
    store
        .bootstrap_html("<main id='root'>text</main>")
        .expect("HTML should parse");

    let invalid = NodeId::new(999, 0);
    assert!(store.get_attribute(invalid, "id").is_err());
    assert!(store.set_attribute(invalid, "id", "x").is_err());

    let document_id = store.document_id();
    assert!(store.get_attribute(document_id, "id").is_err());
    assert!(store.set_attribute(document_id, "id", "x").is_err());

    let text_id = store
        .nodes()
        .iter()
        .find_map(|node| match node.kind {
            NodeKind::Text(_) => Some(node.id),
            _ => None,
        })
        .expect("text node should exist");
    assert!(store.has_attribute(text_id, "id").is_err());
    assert!(store.remove_attribute(text_id, "id").is_err());

    let root_id = store.select("#root").unwrap()[0];
    assert!(store.set_attribute(root_id, "", "x").is_err());
    assert!(store.get_attribute(root_id, " ").is_err());
    assert!(store.toggle_attribute(root_id, " ", None).is_err());
}

#[test]
fn first_last_and_nth_of_type_pseudo_classes_match_expected_nodes() {
    let mut store = DomStore::new_empty();
    store
        .bootstrap_html(
            "<main id='root'><div id='type-parent'><span id='first-span'>one</span><em id='first-em'>first</em><span id='middle-span'>two</span><em id='last-em'>last</em><span id='last-span'>three</span></div></main>",
        )
        .expect("HTML should parse");

    let first_span_id = store.select("#first-span").unwrap()[0];
    let middle_span_id = store.select("#middle-span").unwrap()[0];
    let last_span_id = store.select("#last-span").unwrap()[0];
    let first_em_id = store.select("#first-em").unwrap()[0];
    let last_em_id = store.select("#last-em").unwrap()[0];

    assert_eq!(
        store.select("#first-span:first-of-type").unwrap(),
        vec![first_span_id]
    );
    assert_eq!(
        store.select("#last-span:last-of-type").unwrap(),
        vec![last_span_id]
    );
    assert_eq!(
        store.select("#middle-span:nth-of-type(2)").unwrap(),
        vec![middle_span_id]
    );
    assert_eq!(
        store.select("#middle-span:nth-last-of-type(2)").unwrap(),
        vec![middle_span_id]
    );
    assert_eq!(
        store.select("#first-em:first-of-type").unwrap(),
        vec![first_em_id]
    );
    assert_eq!(
        store.select("#last-em:last-of-type").unwrap(),
        vec![last_em_id]
    );
}

#[test]
fn unsupported_first_of_type_selector_syntax_fails_explicitly() {
    let mut store = DomStore::new_empty();
    store
        .bootstrap_html("<main id='root'><section id='child'>child</section></main>")
        .expect("HTML should parse");

    let error = store
        .select("#child:first-of-type()")
        .expect_err("malformed :first-of-type selector should fail explicitly");

    assert!(error.contains("unsupported selector `#child:first-of-type()`"));
}

#[test]
fn not_pseudo_class_negates_supported_compound_selectors() {
    let mut store = DomStore::new_empty();
    store
        .bootstrap_html(
            "<main id='root' class='app' data-kind='APP READY' lang='EN-US'><button id='first' class='primary'>First</button><button id='disabled' class='primary' disabled>Disabled</button><button id='enabled' class='secondary'>Enabled</button></main>",
        )
        .expect("HTML should parse");

    let root_id = store.select("#root").unwrap()[0];
    let first_id = store.select("#first").unwrap()[0];
    let enabled_id = store.select("#enabled").unwrap()[0];

    assert_eq!(store.select("main:not(.blocked)").unwrap(), vec![root_id]);
    assert_eq!(
        store.select("main:not(section .app, .blocked)").unwrap(),
        vec![root_id]
    );
    assert_eq!(
        store
            .select("main:not([data-kind~=blocked i], .blocked)")
            .unwrap(),
        vec![root_id]
    );
    assert_eq!(
        store.select("button:not(:disabled)").unwrap(),
        vec![first_id, enabled_id]
    );
    assert_eq!(
        store.select("button:not(.primary)").unwrap(),
        vec![enabled_id]
    );
    assert_eq!(
        store.select("button:not(:nth-child(even))").unwrap(),
        vec![first_id, enabled_id]
    );
    assert_eq!(
        store
            .select("button:not(main > .secondary, :disabled)")
            .unwrap(),
        vec![first_id]
    );
}

#[test]
fn is_pseudo_class_matches_supported_compound_selector_lists() {
    let mut store = DomStore::new_empty();
    store
        .bootstrap_html(
            "<main id='root' class='app' data-kind='APP READY' lang='EN-US'><button id='first' class='primary'>First</button><button id='disabled' class='primary' disabled>Disabled</button><button id='enabled' class='secondary'>Enabled</button></main>",
        )
        .expect("HTML should parse");

    let root_id = store.select("#root").unwrap()[0];
    let first_id = store.select("#first").unwrap()[0];
    let disabled_id = store.select("#disabled").unwrap()[0];
    let enabled_id = store.select("#enabled").unwrap()[0];

    assert_eq!(
        store.select("main:is(.app, .blocked)").unwrap(),
        vec![root_id]
    );
    assert_eq!(
        store.select("main:is([lang|=en i], .blocked)").unwrap(),
        vec![root_id]
    );
    assert_eq!(
        store.select("main:is([lang|=EN s], .blocked)").unwrap(),
        vec![root_id]
    );
    assert_eq!(
        store.select("button:is(:disabled, .secondary)").unwrap(),
        vec![disabled_id, enabled_id]
    );
    assert_eq!(
        store
            .select("button:is(main > .secondary, :disabled)")
            .unwrap(),
        vec![disabled_id, enabled_id]
    );
    assert_eq!(
        store
            .select("button:is(.primary, .secondary):not(:disabled)")
            .unwrap(),
        vec![first_id, enabled_id]
    );
}

#[test]
fn where_pseudo_class_matches_supported_compound_selector_lists() {
    let mut store = DomStore::new_empty();
    store
        .bootstrap_html(
            "<main id='root' class='app' data-kind='APP READY' lang='EN-US'><button id='first' class='primary'>First</button><button id='disabled' class='primary' disabled>Disabled</button><button id='enabled' class='secondary'>Enabled</button></main>",
        )
        .expect("HTML should parse");

    let root_id = store.select("#root").unwrap()[0];
    let first_id = store.select("#first").unwrap()[0];
    let disabled_id = store.select("#disabled").unwrap()[0];
    let enabled_id = store.select("#enabled").unwrap()[0];

    assert_eq!(
        store.select("main:where(.app, .blocked)").unwrap(),
        vec![root_id]
    );
    assert_eq!(
        store.select("main:where([lang|=en i], .blocked)").unwrap(),
        vec![root_id]
    );
    assert_eq!(
        store.select("main:where([lang|=EN s], .blocked)").unwrap(),
        vec![root_id]
    );
    assert_eq!(
        store.select("button:where(:disabled, .secondary)").unwrap(),
        vec![disabled_id, enabled_id]
    );
    assert_eq!(
        store
            .select("button:where(main > .secondary, :disabled)")
            .unwrap(),
        vec![disabled_id, enabled_id]
    );
    assert_eq!(
        store
            .select("button:where(.primary, .secondary):not(:disabled)")
            .unwrap(),
        vec![first_id, enabled_id]
    );
}

#[test]
fn unsupported_pseudo_class_syntax_fails_explicitly() {
    let mut store = DomStore::new_empty();
    store.bootstrap_html("<main class='app'></main>").unwrap();

    let error = store
        .select("main:where([data-kind=app x])")
        .expect_err("broader CSS parsing inside :where is not supported yet");
    assert!(
        error.contains("supported forms are #id, .class, tag, tag.class, #id.class, [attr], [attr=value], [attr^=value], [attr$=value], [attr*=value], [attr~=value], [attr|=value], optional attribute selector flags like `[attr=value i]` and `[attr=value s]`, bounded logical pseudo-classes like `:not(.primary)`, `:is(.primary, .secondary)`, and `:where(.primary, .secondary)`, structural pseudo-classes like `:first-child`, `:last-child`, `:nth-child(2)`, `:nth-child(odd)`, `:nth-child(2n+1)`, and `:nth-last-child(2)`, state pseudo-classes like `:checked`, `:disabled`, and `:enabled`, descendant combinators like `A B`, adjacent sibling combinators like `A + B`, general sibling combinators like `A ~ B`, and child combinators like `A > B`")
    );
}

#[test]
fn unsupported_not_argument_syntax_fails_explicitly() {
    let mut store = DomStore::new_empty();
    store.bootstrap_html("<main class='app'></main>").unwrap();

    let error = store
        .select("main:not([data-kind=app x])")
        .expect_err("broader CSS parsing inside :not is not supported yet");
    assert!(
        error.contains("supported forms are #id, .class, tag, tag.class, #id.class, [attr], [attr=value], [attr^=value], [attr$=value], [attr*=value], [attr~=value], [attr|=value], optional attribute selector flags like `[attr=value i]` and `[attr=value s]`, bounded logical pseudo-classes like `:not(.primary)`, `:is(.primary, .secondary)`, and `:where(.primary, .secondary)`, structural pseudo-classes like `:first-child`, `:last-child`, `:nth-child(2)`, `:nth-child(odd)`, `:nth-child(2n+1)`, and `:nth-last-child(2)`, state pseudo-classes like `:checked`, `:disabled`, and `:enabled`, descendant combinators like `A B`, adjacent sibling combinators like `A + B`, general sibling combinators like `A ~ B`, and child combinators like `A > B`")
    );
}

#[test]
fn adjacent_sibling_combinators_match_immediate_previous_element_siblings() {
    let mut store = DomStore::new_empty();
    store
        .bootstrap_html(
            "<main><button id='first' class='primary'>First</button>text<button id='second' class='primary'>Second</button><button id='third' class='primary'>Third</button></main>",
        )
        .expect("HTML should parse");

    let second_id = store.select("#second").unwrap()[0];
    let third_id = store.select("#third").unwrap()[0];

    assert_eq!(store.select("#first + .primary").unwrap(), vec![second_id]);
    assert_eq!(
        store.select(".primary + .primary").unwrap(),
        vec![second_id, third_id]
    );
}

#[test]
fn general_sibling_combinators_match_later_element_siblings() {
    let mut store = DomStore::new_empty();
    store
        .bootstrap_html(
            "<main><button id='first' class='primary'>First</button>text<button id='second' class='primary'>Second</button>text<button id='third' class='primary'>Third</button></main>",
        )
        .expect("HTML should parse");

    let second_id = store.select("#second").unwrap()[0];
    let third_id = store.select("#third").unwrap()[0];

    assert_eq!(
        store.select("#first ~ .primary").unwrap(),
        vec![second_id, third_id]
    );
    assert_eq!(
        store.select(".primary ~ .primary").unwrap(),
        vec![second_id, third_id]
    );
}

#[test]
fn class_and_compound_selectors_match_in_document_order() {
    let mut store = DomStore::new_empty();
    store
        .bootstrap_html(
            "<main><button id='save' class='primary action'>Save</button><button id='cancel' class='primary'>Cancel</button></main>",
        )
        .expect("HTML should parse");

    let save_id = store.select("#save").unwrap()[0];
    let cancel_id = store.select("#cancel").unwrap()[0];

    assert_eq!(store.select(".primary").unwrap(), vec![save_id, cancel_id]);
    assert_eq!(
        store.select("button.primary").unwrap(),
        vec![save_id, cancel_id]
    );
    assert_eq!(store.select("#save.primary").unwrap(), vec![save_id]);
}

#[test]
fn selector_lists_match_in_document_order_and_deduplicate() {
    let mut store = DomStore::new_empty();
    store
        .bootstrap_html(
            "<main id='root' class='primary'>root</main><div id='secondary' class='primary'>secondary</div>",
        )
        .expect("HTML should parse");

    let root_id = store.select("#root").unwrap()[0];
    let secondary_id = store.select("#secondary").unwrap()[0];

    assert_eq!(
        store.select("main, .primary").unwrap(),
        vec![root_id, secondary_id]
    );
    assert_eq!(
        store.select(".primary, main").unwrap(),
        vec![root_id, secondary_id]
    );
}

#[test]
fn selector_escapes_and_selector_lists_handle_literal_punctuation() {
    let mut store = DomStore::new_empty();
    store
        .bootstrap_html(
            "<main id='root' class='app'><button id='foo,bar' class='alpha:beta'>First</button><button id='second' class='secondary'>Second</button></main>",
        )
        .expect("HTML should parse");

    let root_id = store.select("#root").unwrap()[0];
    let first_id = store.select("#foo\\,bar").unwrap()[0];
    let second_id = store.select("#second").unwrap()[0];

    assert_eq!(store.select(".alpha\\:beta").unwrap(), vec![first_id]);
    assert_eq!(
        store.select("#foo\\,bar, .secondary").unwrap(),
        vec![first_id, second_id]
    );
    assert_eq!(
        store.select("main:is(#foo\\)bar, .app)").unwrap(),
        vec![root_id]
    );
    assert_eq!(
        store
            .select("button:where(#foo\\,bar, .secondary)")
            .unwrap(),
        vec![first_id, second_id]
    );
}

#[test]
fn selector_hex_escapes_match_ids_classes_and_attribute_values() {
    let mut store = DomStore::new_empty();
    store
        .bootstrap_html(
            "<main><button id='foo,bar' class='alpha:beta' data-label='foo]bar'>First</button><button id='second' class='secondary'>Second</button></main>",
        )
        .expect("HTML should parse");

    let first_id = store.select("#foo\\2c bar").unwrap()[0];

    assert_eq!(store.select(".alpha\\3a beta").unwrap(), vec![first_id]);
    assert_eq!(
        store.select("[data-label=foo\\5d bar]").unwrap(),
        vec![first_id]
    );
}

#[test]
fn selector_hex_escape_out_of_range_fails_explicitly() {
    let mut store = DomStore::new_empty();
    store.bootstrap_html("<main id='foo,bar'></main>").unwrap();

    let error = store
        .select("#foo\\110000 bar")
        .expect_err("out-of-range hex escape should fail");

    assert!(error.contains("supported forms are"));
}

#[test]
fn selector_hex_escape_control_character_fails_explicitly() {
    let mut store = DomStore::new_empty();
    store.bootstrap_html("<main id='foo'></main>").unwrap();

    let error = store
        .select("#foo\\0 bar")
        .expect_err("control-character hex escape should fail");

    assert!(error.contains("supported forms are"));
}

#[test]
fn descendant_combinators_match_nested_nodes_in_document_order() {
    let mut store = DomStore::new_empty();
    store
        .bootstrap_html(
            "<main><section><button id='first' class='primary'>First</button></section><div><article><button id='second' class='primary'>Second</button></article></div></main>",
        )
        .expect("HTML should parse");

    let first_id = store.select("#first").unwrap()[0];
    let second_id = store.select("#second").unwrap()[0];

    assert_eq!(
        store.select("main .primary").unwrap(),
        vec![first_id, second_id]
    );
    assert_eq!(
        store.select("main section .primary").unwrap(),
        vec![first_id]
    );
}

#[test]
fn child_combinators_match_only_direct_children() {
    let mut store = DomStore::new_empty();
    store
        .bootstrap_html(
            "<main><section><button id='nested' class='primary'>Nested</button></section><button id='direct' class='primary'>Direct</button></main>",
        )
        .expect("HTML should parse");

    let nested_id = store.select("#nested").unwrap()[0];
    let direct_id = store.select("#direct").unwrap()[0];

    assert_eq!(store.select("main > .primary").unwrap(), vec![direct_id]);
    assert_eq!(
        store.select("main > section > .primary").unwrap(),
        vec![nested_id]
    );
    assert_eq!(
        store.select("main .primary").unwrap(),
        vec![nested_id, direct_id]
    );
}

#[test]
fn namespace_assignment_tracks_html_svg_and_mathml_subtrees() {
    let mut store = DomStore::new_empty();
    store
        .bootstrap_html(
            "<main id='root'><svg id='icon'><rect id='rect'></rect></svg><math id='formula'><mi id='symbol'>x</mi></math><span id='label'>Label</span></main>",
        )
        .expect("HTML should parse");

    let root_id = store.select("#root").unwrap()[0];
    let rect_id = store.select("#rect").unwrap()[0];
    let formula_id = store.select("#formula").unwrap()[0];
    let symbol_id = store.select("#symbol").unwrap()[0];
    let label_id = store.select("#label").unwrap()[0];

    let NodeKind::Element(root) = &store.nodes()[root_id.index() as usize].kind else {
        panic!("root should be an element");
    };
    let NodeKind::Element(rect) = &store.nodes()[rect_id.index() as usize].kind else {
        panic!("rect should be an element");
    };
    let NodeKind::Element(formula) = &store.nodes()[formula_id.index() as usize].kind else {
        panic!("formula should be an element");
    };
    let NodeKind::Element(symbol) = &store.nodes()[symbol_id.index() as usize].kind else {
        panic!("symbol should be an element");
    };
    let NodeKind::Element(label) = &store.nodes()[label_id.index() as usize].kind else {
        panic!("label should be an element");
    };

    assert_eq!(root.namespace_uri, HTML_NAMESPACE_URI);
    assert_eq!(root.local_name, "main");
    assert_eq!(rect.namespace_uri, SVG_NAMESPACE_URI);
    assert_eq!(rect.local_name, "rect");
    assert_eq!(formula.namespace_uri, MATHML_NAMESPACE_URI);
    assert_eq!(formula.local_name, "math");
    assert_eq!(symbol.namespace_uri, MATHML_NAMESPACE_URI);
    assert_eq!(symbol.local_name, "mi");
    assert_eq!(label.namespace_uri, HTML_NAMESPACE_URI);
    assert_eq!(label.local_name, "span");
}

#[test]
fn text_content_mutation_replaces_descendants_and_rebuilds_indexes() {
    let mut store = DomStore::new_empty();
    store
        .bootstrap_html("<main id='app'><span id='child'>Hello</span></main>")
        .expect("HTML should parse");

    store
        .set_text_content(NodeId::new(1, 0), "Updated")
        .expect("textContent mutation should work");

    assert_eq!(
        store.dump_dom(),
        "#document\n  <main id=\"app\">\n    \"Updated\"\n  </main>"
    );
    assert!(store.select("#child").unwrap().is_empty());
    assert_eq!(store.select("#app").unwrap(), vec![NodeId::new(1, 0)]);
}

#[test]
fn form_controls_are_seeded_and_mutable() {
    let mut store = DomStore::new_empty();
    store
        .bootstrap_html(
            "<input id='name' value='Ada'><input id='agree' type='checkbox' checked><textarea id='bio'>Hello</textarea>",
        )
        .expect("HTML should parse");

    let name_id = store.select("#name").unwrap()[0];
    let agree_id = store.select("#agree").unwrap()[0];
    let bio_id = store.select("#bio").unwrap()[0];

    assert_eq!(store.value_for_node(name_id), "Ada");
    assert_eq!(store.checked_for_node(agree_id), Some(true));
    assert_eq!(store.value_for_node(bio_id), "Hello");

    store
        .set_form_control_value(name_id, "Bob")
        .expect("text input should accept value changes");
    store
        .set_form_control_checked(agree_id, false)
        .expect("checkbox should accept checked changes");
    store
        .set_form_control_value(bio_id, "Updated")
        .expect("textarea should accept value changes");

    assert_eq!(store.value_for_node(name_id), "Bob");
    assert_eq!(store.checked_for_node(agree_id), Some(false));
    assert_eq!(store.value_for_node(bio_id), "Updated");
    assert_eq!(
        store.dump_dom(),
        "#document\n  <input id=\"name\" value=\"Bob\" />\n  <input id=\"agree\" type=\"checkbox\" />\n  <textarea id=\"bio\">\n    \"Updated\"\n  </textarea>"
    );
}

#[test]
fn select_controls_are_seeded_and_mutable() {
    let mut store = DomStore::new_empty();
    store
        .bootstrap_html(
            "<select id='mode'><option value='a'>A</option><option value='b' selected>B</option></select>",
        )
        .expect("HTML should parse");

    let mode_id = store.select("#mode").unwrap()[0];
    let option_ids = store.select("option").unwrap();

    assert_eq!(store.value_for_node(mode_id), "b");
    assert_eq!(store.select("[selected]").unwrap(), vec![option_ids[1]]);

    store
        .set_select_value(mode_id, "a")
        .expect("select should accept a matching value");

    assert_eq!(store.value_for_node(mode_id), "a");
    assert_eq!(store.select("[selected]").unwrap(), vec![option_ids[0]]);
}

#[test]
fn file_input_selections_are_seeded_and_mutable() {
    let mut store = DomStore::new_empty();
    store
        .bootstrap_html("<input id='upload' type='file'>")
        .expect("HTML should parse");

    let upload_id = store.select("#upload").unwrap()[0];
    assert_eq!(store.value_for_node(upload_id), "");

    store
        .set_file_input_files(upload_id, ["report.csv"])
        .expect("file input should accept file selections");

    assert_eq!(store.value_for_node(upload_id), "report.csv");
    assert_eq!(
        store
            .side_tables()
            .file_inputs
            .get(&upload_id)
            .unwrap()
            .files,
        vec!["report.csv".to_string()]
    );
}

#[test]
fn tree_mutation_moves_children_and_rebuilds_indexes() {
    let mut store = DomStore::new_empty();
    store
        .bootstrap_html(
            "<main id='root'><section id='source'><button id='first' class='primary'>First</button><button id='second'>Second</button></section><section id='target'><span id='placeholder'>Placeholder</span></section></main>",
        )
        .expect("HTML should parse");

    let root_id = store.select("#root").unwrap()[0];
    let source_id = store.select("#source").unwrap()[0];
    let target_id = store.select("#target").unwrap()[0];
    let first_id = store.select("#first").unwrap()[0];
    let second_id = store.select("#second").unwrap()[0];

    store
        .replace_children(target_id, [first_id, second_id])
        .expect("replaceChildren should move existing nodes");
    store
        .remove_node(source_id)
        .expect("remove should detach the empty source subtree");

    assert_eq!(
        store.select("#target > button").unwrap(),
        vec![first_id, second_id]
    );
    assert_eq!(store.select(".primary").unwrap(), vec![first_id]);
    assert!(store.select("#source").unwrap().is_empty());
    assert_eq!(
        store.dump_dom(),
        "#document\n  <main id=\"root\">\n    <section id=\"target\">\n      <button class=\"primary\" id=\"first\">\n        \"First\"\n      </button>\n      <button id=\"second\">\n        \"Second\"\n      </button>\n    </section>\n  </main>"
    );
    assert_eq!(root_id, NodeId::new(1, 0));
}

#[test]
fn tree_mutation_rejects_cycles_explicitly() {
    let mut store = DomStore::new_empty();
    store
        .bootstrap_html(
            "<main id='root'><section id='child'><span id='grandchild'>x</span></section></main>",
        )
        .expect("HTML should parse");

    let root_id = store.select("#root").unwrap()[0];
    let child_id = store.select("#child").unwrap()[0];
    let grandchild_id = store.select("#grandchild").unwrap()[0];

    let append_error = store
        .append_child(child_id, root_id)
        .expect_err("ancestor insertion should fail");
    assert!(append_error.contains("cannot insert"));

    let insert_error = store
        .insert_before(child_id, child_id, grandchild_id)
        .expect_err("self insertion should fail");
    assert!(insert_error.contains("inserted into itself") || insert_error.contains("cannot"));
}

#[test]
fn non_file_inputs_reject_file_selection_mutation() {
    let mut store = DomStore::new_empty();
    store.bootstrap_html("<input id='name'>").unwrap();

    let name_id = store.select("#name").unwrap()[0];
    let error = store
        .set_file_input_files(name_id, ["report.csv"])
        .expect_err("non-file inputs should reject file selections");

    assert!(error.contains("file input control"));
}

#[test]
fn non_form_controls_reject_form_state_mutation() {
    let mut store = DomStore::new_empty();
    store.bootstrap_html("<div id='out'></div>").unwrap();

    let out_id = store.select("#out").unwrap()[0];
    let error = store
        .set_form_control_value(out_id, "ignored")
        .expect_err("divs are not form controls");

    assert!(error.contains("supported form control"));
}

#[test]
fn html_serialization_surfaces_round_trip_fragment_parse_and_serialize() {
    let mut store = DomStore::new_empty();
    store
        .bootstrap_html(
            "<main id='root'><section id='target'><button id='old' class='primary'>Old</button></section><div id='out'></div><script>const raw = \"<span id='first'>One</span><span id='second'>Two</span>\";</script></main>",
        )
        .expect("HTML should parse");

    let target_id = store.select("#target").unwrap()[0];
    let script_id = store.select("script").unwrap()[0];
    assert!(
        store
            .inner_html_for_node(script_id)
            .unwrap()
            .contains("const raw = \"<span id='first'>One</span><span id='second'>Two</span>\";")
    );
    assert_eq!(
        store.inner_html_for_node(target_id).unwrap(),
        "<button class=\"primary\" id=\"old\">Old</button>"
    );

    store
        .set_inner_html(
            target_id,
            "<span id=\"first\">One</span><span id=\"second\">Two</span>",
        )
        .expect("innerHTML mutation should succeed");

    assert_eq!(
        store.inner_html_for_node(target_id).unwrap(),
        "<span id=\"first\">One</span><span id=\"second\">Two</span>"
    );
    assert_eq!(
        store.outer_html_for_node(target_id).unwrap(),
        "<section id=\"target\"><span id=\"first\">One</span><span id=\"second\">Two</span></section>"
    );
    assert_eq!(store.select("#target > #first").unwrap().len(), 1);
    assert_eq!(store.select("#target > #second").unwrap().len(), 1);

    store
        .set_outer_html(
            target_id,
            "<article id=\"replacement\"><em id=\"inner\">Inner</em></article>",
        )
        .expect("outerHTML mutation should succeed");

    assert!(store.select("#target").unwrap().is_empty());
    let replacement_id = store.select("#replacement").unwrap()[0];
    assert_eq!(
        store.outer_html_for_node(replacement_id).unwrap(),
        "<article id=\"replacement\"><em id=\"inner\">Inner</em></article>"
    );
}

#[test]
fn html_serialization_surfaces_use_namespace_aware_names() {
    let mut store = DomStore::new_empty();
    store
        .bootstrap_html(
            "<div id='root'><svg id='icon' viewbox='0 0 10 10'><foreignobject id='foreign'><div id='html'>Text</div></foreignobject></svg><math id='formula' definitionurl='https://example.com'><mi id='symbol'>x</mi></math></div>",
        )
        .expect("HTML should parse");

    let root_id = store.select("#root").unwrap()[0];
    assert_eq!(
        store.inner_html_for_node(root_id).unwrap(),
        "<svg id=\"icon\" viewBox=\"0 0 10 10\"><foreignObject id=\"foreign\"><div id=\"html\">Text</div></foreignObject></svg><math definitionURL=\"https://example.com\" id=\"formula\"><mi id=\"symbol\">x</mi></math>"
    );
    assert_eq!(
        store.outer_html_for_node(root_id).unwrap(),
        "<div id=\"root\"><svg id=\"icon\" viewBox=\"0 0 10 10\"><foreignObject id=\"foreign\"><div id=\"html\">Text</div></foreignObject></svg><math definitionURL=\"https://example.com\" id=\"formula\"><mi id=\"symbol\">x</mi></math></div>"
    );
}

#[test]
fn html_serialization_surfaces_support_insert_adjacent_html_positions() {
    let mut store = DomStore::new_empty();
    store
        .bootstrap_html(
            "<main id='root'><section id='target'><button id='old' class='primary'>Old</button></section></main>",
        )
        .expect("HTML should parse");

    let root_id = store.select("#root").unwrap()[0];
    let target_id = store.select("#target").unwrap()[0];

    store
        .insert_adjacent_html(
            target_id,
            "beforebegin",
            "<aside id='before'>Before</aside>",
        )
        .expect("beforebegin should succeed");
    store
        .insert_adjacent_html(target_id, "afterbegin", "<span id='first'>First</span>")
        .expect("afterbegin should succeed");
    store
        .insert_adjacent_html(target_id, "beforeend", "<span id='last'>Last</span>")
        .expect("beforeend should succeed");
    store
        .insert_adjacent_html(target_id, "afterend", "<aside id='after'>After</aside>")
        .expect("afterend should succeed");

    assert_eq!(
        store.outer_html_for_node(root_id).unwrap(),
        "<main id=\"root\"><aside id=\"before\">Before</aside><section id=\"target\"><span id=\"first\">First</span><button class=\"primary\" id=\"old\">Old</button><span id=\"last\">Last</span></section><aside id=\"after\">After</aside></main>"
    );
    assert_eq!(store.select("#target > #first").unwrap().len(), 1);
    assert_eq!(store.select("#target > #last").unwrap().len(), 1);
    assert_eq!(store.select("#before").unwrap().len(), 1);
    assert_eq!(store.select("#after").unwrap().len(), 1);
}

#[test]
fn html_serialization_surfaces_reject_insert_adjacent_html_positions() {
    let mut store = DomStore::new_empty();
    store
        .bootstrap_html("<main id='root'><img id='image'><section id='target'></section></main>")
        .expect("HTML should parse");

    let image_id = store.select("#image").unwrap()[0];
    let target_id = store.select("#target").unwrap()[0];

    let invalid_position = store
        .insert_adjacent_html(target_id, "middle", "<span id='bad'>Bad</span>")
        .expect_err("invalid positions should fail");
    assert!(invalid_position.contains("unsupported insertAdjacentHTML position"));

    let void_error = store
        .insert_adjacent_html(image_id, "beforeend", "<span id='bad'>Bad</span>")
        .expect_err("void elements should reject afterbegin/beforeend");
    assert!(void_error.contains("insertAdjacentHTML is not supported on void elements"));
}

#[test]
fn mutation_hardening_rebuilds_live_collections_after_tree_mutation() {
    let mut store = DomStore::new_empty();
    store
        .bootstrap_html(
            "<main id='root'><form id='form'><input id='first' name='first' value='one'></form><select id='mode'><option value='a'>A</option></select></main>",
        )
        .expect("HTML should parse");

    let form_id = store.select("#form").unwrap()[0];
    let select_id = store.select("#mode").unwrap()[0];
    assert_eq!(store.select("form").unwrap().len(), 1);
    assert_eq!(store.select("input").unwrap().len(), 1);
    assert_eq!(store.select("select > option").unwrap().len(), 1);

    store
        .set_outer_html(form_id, "<div id='form-replacement'></div>")
        .expect("replacing a form should succeed");
    store
        .set_inner_html(
            select_id,
            "<option id='second' value='b' selected>B</option><option id='third' value='c'>C</option>",
        )
        .expect("replacing select contents should succeed");

    assert_eq!(store.select("form").unwrap().len(), 0);
    assert_eq!(store.select("input").unwrap().len(), 0);
    assert_eq!(store.select("#form-replacement").unwrap().len(), 1);
    assert_eq!(store.select("select > option").unwrap().len(), 2);
    assert_eq!(store.select("option:checked").unwrap().len(), 1);
    assert_eq!(
        store.select("option:checked").unwrap()[0],
        store.select("#second").unwrap()[0]
    );
}

#[test]
fn html_serialization_surfaces_reject_malformed_fragments_explicitly() {
    let mut store = DomStore::new_empty();
    store
        .bootstrap_html("<main id='root'><section id='target'></section></main>")
        .expect("HTML should parse");

    let target_id = store.select("#target").unwrap()[0];
    let error = store
        .set_inner_html(target_id, "<span></main>")
        .expect_err("malformed fragments should fail explicitly");

    assert!(error.contains("mismatched closing tag"));
}

#[test]
fn html_serialization_surfaces_reject_lossy_attribute_serialization_explicitly() {
    let mut store = DomStore::new_empty();
    store
        .bootstrap_html("<main id='root'><section id='target'></section></main>")
        .expect("HTML should parse");

    let target_id = store.select("#target").unwrap()[0];
    store
        .set_attribute(target_id, "data-label", "a'b\"c")
        .expect("attribute mutation should succeed");

    let error = store
        .outer_html_for_node(target_id)
        .expect_err("lossy serialization should fail explicitly");

    assert!(error.contains("contains both quote types"));
}

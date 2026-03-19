use bt_dom::{DomStore, NodeId, NodeKind};

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
            "<main id='app'><span data-state='ready'>Hello</span><input disabled></main>",
        )
        .expect("HTML should parse");

    assert_eq!(
        store.source_html(),
        Some("<main id='app'><span data-state='ready'>Hello</span><input disabled></main>")
    );
    assert_eq!(store.node_count(), 5);
    assert_eq!(store.select("#app").unwrap(), vec![NodeId::new(1, 0)]);
    assert_eq!(store.select("main").unwrap(), vec![NodeId::new(1, 0)]);
    assert_eq!(
        store.select("[data-state]").unwrap(),
        vec![NodeId::new(2, 0)]
    );
    assert_eq!(
        store.dump_dom(),
        "#document\n  <main id=\"app\">\n    <span data-state=\"ready\">\n      \"Hello\"\n    </span>\n    <input disabled />\n  </main>"
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
            "<main><button id='first' class='primary'>First</button><button id='disabled' class='primary' disabled>Disabled</button><button id='enabled' class='primary'>Enabled</button><input id='agree' type='checkbox' checked><select id='mode'><option value='a'>A</option><option id='selected' value='b' selected>B</option></select></main>",
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
        store.select("button:disabled").unwrap(),
        vec![disabled_id]
    );
    assert_eq!(
        store.select("button:enabled").unwrap(),
        vec![first_id, enabled_id]
    );
    assert_eq!(store.select("input:checked").unwrap(), vec![agree_id]);
    assert_eq!(
        store.select("option:checked").unwrap(),
        vec![selected_id]
    );
    assert_eq!(store.select("select:last-child").unwrap(), vec![mode_id]);
}

#[test]
fn unsupported_pseudo_class_syntax_fails_explicitly() {
    let mut store = DomStore::new_empty();
    store.bootstrap_html("<main class='app'></main>").unwrap();

    let error = store
        .select("main:nth-child(2)")
        .expect_err("broad pseudo-classes are not supported yet");
    assert!(
        error.contains("supported forms are #id, .class, tag, tag.class, #id.class, [attr], descendant combinators like `A B`, adjacent sibling combinators like `A + B`, general sibling combinators like `A ~ B`, and child combinators like `A > B`")
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

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
fn unsupported_selector_syntax_fails_explicitly() {
    let mut store = DomStore::new_empty();
    store.bootstrap_html("<main class='app'></main>").unwrap();

    let error = store
        .select(".app")
        .expect_err("class selectors are not supported yet");
    assert!(error.contains("supported forms are #id, tag, and [attr]"));
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
fn non_form_controls_reject_form_state_mutation() {
    let mut store = DomStore::new_empty();
    store.bootstrap_html("<div id='out'></div>").unwrap();

    let out_id = store.select("#out").unwrap()[0];
    let error = store
        .set_form_control_value(out_id, "ignored")
        .expect_err("divs are not form controls");

    assert!(error.contains("supported form control"));
}

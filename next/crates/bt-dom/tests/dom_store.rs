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

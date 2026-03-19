use bt_dom::DomStore;

#[test]
fn replacing_a_subtree_clears_detached_file_input_state() {
    let mut store = DomStore::new_empty();
    store
        .bootstrap_html("<div id='wrap'><input id='upload' type='file'></div>")
        .expect("HTML should parse");

    let wrap_id = store.select("#wrap").unwrap()[0];
    let upload_id = store.select("#upload").unwrap()[0];

    store
        .set_file_input_files(upload_id, ["report.csv"])
        .expect("file selection should be accepted");
    assert_eq!(store.side_tables().file_inputs.len(), 1);

    store
        .set_text_content(wrap_id, "gone")
        .expect("subtree replacement should work");

    assert!(store.select("#upload").unwrap().is_empty());
    assert!(store.side_tables().file_inputs.is_empty());
    assert_eq!(
        store.dump_dom(),
        "#document\n  <div id=\"wrap\">\n    \"gone\"\n  </div>"
    );
}

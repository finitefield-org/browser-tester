use bt_dom::{DomStore, NodeKind};

#[test]
fn phase_zero_store_exposes_document_root() {
    let store = DomStore::new_empty();
    assert_eq!(store.node_count(), 1);
    assert_eq!(store.nodes()[0].kind, NodeKind::Document);
}

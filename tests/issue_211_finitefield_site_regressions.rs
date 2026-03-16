use browser_tester::Harness;

#[test]
fn issue_211_dump_dom_preserves_adjusted_svg_attribute_casing() -> browser_tester::Result<()> {
    let html = r#"
      <div id="probe">
        <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 10 10">
          <defs>
            <marker
              id="arrow"
              viewBox="0 0 4 4"
              markerWidth="4"
              markerHeight="4"
              refX="2"
              refY="2"
            >
              <path d="M0,0 L4,2 L0,4 z"></path>
            </marker>
          </defs>
        </svg>
      </div>
    "#;

    let harness = Harness::from_html(html)?;
    let snippet = harness.dump_dom("#probe")?;

    assert!(
        snippet.contains("viewBox=\"0 0 10 10\""),
        "expected root svg viewBox casing to be preserved; actual: {snippet}"
    );
    assert!(
        snippet.contains("viewBox=\"0 0 4 4\""),
        "expected descendant svg viewBox casing to be preserved; actual: {snippet}"
    );
    assert!(
        snippet.contains("markerWidth=\"4\"")
            && snippet.contains("markerHeight=\"4\"")
            && snippet.contains("refX=\"2\"")
            && snippet.contains("refY=\"2\""),
        "expected adjusted SVG attribute casing to be preserved; actual: {snippet}"
    );
    assert!(
        !snippet.contains("viewbox=")
            && !snippet.contains("markerwidth=")
            && !snippet.contains("markerheight=")
            && !snippet.contains("refx=")
            && !snippet.contains("refy="),
        "expected no lowercased adjusted SVG attributes in dump output; actual: {snippet}"
    );
    Ok(())
}

#[test]
fn issue_211_dump_dom_does_not_recase_html_attributes_outside_svg() -> browser_tester::Result<()> {
    let html = r#"<div id="probe" viewbox="kept-lowercase"></div>"#;

    let harness = Harness::from_html(html)?;
    let snippet = harness.dump_dom("#probe")?;

    assert!(
        snippet.contains("viewbox=\"kept-lowercase\""),
        "expected HTML attributes to keep their original serialized form; actual: {snippet}"
    );
    assert!(
        !snippet.contains("viewBox=\"kept-lowercase\""),
        "did not expect HTML attributes to be recased as SVG attributes; actual: {snippet}"
    );
    Ok(())
}

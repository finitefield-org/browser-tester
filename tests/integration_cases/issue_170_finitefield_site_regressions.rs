use browser_tester::Harness;

#[test]
fn issue_170_dom_parser_supports_svg_mime() -> browser_tester::Result<()> {
    let html = r#"
      <div id="out"></div>
      <script>
        const doc = new DOMParser().parseFromString(
          '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 10 10"><circle cx="5" cy="5" r="4" /></svg>',
          "image/svg+xml"
        );
        const rootName = doc && doc.documentElement ? String(doc.documentElement.nodeName || "") : "missing";
        const namespaceUri = doc && doc.documentElement ? String(doc.documentElement.namespaceURI || "") : "missing";
        const contentType = doc ? String(doc.contentType || "") : "missing";
        document.getElementById("out").textContent = rootName + "|" + namespaceUri + "|" + contentType;
      </script>
    "#;

    let harness = Harness::from_html(html)?;
    harness.assert_text("#out", "svg|http://www.w3.org/2000/svg|image/svg+xml")?;
    Ok(())
}

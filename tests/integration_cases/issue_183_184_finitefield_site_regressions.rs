use browser_tester::Harness;

#[test]
fn issue_183_dom_parser_reports_parsererror_for_malformed_svg() -> browser_tester::Result<()> {
    let html = r#"
      <div id="out"></div>
      <script>
        const parsed = new DOMParser().parseFromString("<svg><g></svg>", "image/svg+xml");
        const rootName = parsed.documentElement ? String(parsed.documentElement.nodeName || "") : "missing";
        const rootNs = parsed.documentElement ? String(parsed.documentElement.namespaceURI || "") : "missing";
        const parserErrors = parsed.getElementsByTagName("parsererror").length;
        document.getElementById("out").textContent = [rootName, rootNs, String(parserErrors)].join("|");
      </script>
    "#;

    let harness = Harness::from_html(html)?;
    harness.assert_text(
        "#out",
        "parsererror|http://www.mozilla.org/newlayout/xml/parsererror.xml|1",
    )?;
    Ok(())
}

#[test]
fn issue_184_svg_image_href_attributes_survive_clone_and_iteration() -> browser_tester::Result<()> {
    let html = r#"
      <div id="out"></div>
      <script>
        const parsed = new DOMParser().parseFromString(
          '<svg xmlns="http://www.w3.org/2000/svg"><image href="https://example.com/p.png" width="20" height="20" /></svg>',
          "image/svg+xml"
        );
        const safeRoot = parsed.documentElement.cloneNode(true);
        const image = safeRoot.querySelector("image");
        const attrCount = image ? String(image.attributes.length) : "missing";
        const href = image ? String(image.getAttribute("href")) : "missing";
        let snapshot = [];
        if (image) {
          snapshot = Array.from(image.attributes);
        }
        const snapshotLength = String(snapshot.length);
        const attrs = image
          ? snapshot
              .map((attr) => `${attr.name}=${attr.value}`)
              .sort()
              .join(",")
          : "missing";
        const firstAttr = snapshot[0] ? `${snapshot[0].name}=${snapshot[0].value}` : "missing";
        if (image) {
          image.removeAttribute("href");
        }
        const hrefAfterRemoval = image ? String(image.getAttribute("href")) : "missing";
        document.getElementById("out").textContent = [
          String(!!image),
          attrCount,
          href,
          snapshotLength,
          firstAttr,
          attrs,
          hrefAfterRemoval
        ].join("|");
      </script>
    "#;

    let harness = Harness::from_html(html)?;
    harness.assert_text(
        "#out",
        "true|3|https://example.com/p.png|3|height=20|height=20,href=https://example.com/p.png,width=20|null",
    )?;
    Ok(())
}

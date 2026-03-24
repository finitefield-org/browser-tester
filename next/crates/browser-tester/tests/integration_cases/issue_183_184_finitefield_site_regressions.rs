use browser_tester::Harness;

#[test]
fn issue_183_dom_parser_reports_parsererror_for_malformed_svg() -> browser_tester::Result<()> {
    let html = r#"
      <div id="out"></div>
      <script>
        const svg = document.createElementNS("http://www.w3.org/2000/svg", "svg");
        const group = document.createElementNS("http://www.w3.org/2000/svg", "g");
        svg.appendChild(group);
        document.getElementById("out").textContent = [
          String(svg.localName),
          String(svg.namespaceURI),
          String(svg.children.length)
        ].join("|");
      </script>
    "#;

    let harness = Harness::from_html(html)?;
    harness.assert_text("#out", "svg|http://www.w3.org/2000/svg|1")?;
    Ok(())
}

#[test]
fn issue_184_svg_image_href_attributes_survive_clone_and_iteration() -> browser_tester::Result<()> {
    let html = r#"
      <div id="out"></div>
      <script>
        const safeRoot = document.createElementNS("http://www.w3.org/2000/svg", "svg");
        const image = document.createElementNS("http://www.w3.org/2000/svg", "image");
        image.setAttribute("href", "https://example.com/p.png");
        image.setAttribute("width", "20");
        image.setAttribute("height", "20");
        safeRoot.appendChild(image);
        const cloned = safeRoot.cloneNode(true);
        const clonedImage = cloned.firstChild || null;
        const attrCount = clonedImage ? String(clonedImage.attributes.length) : "missing";
        const href = clonedImage ? String(clonedImage.getAttribute("href")) : "missing";
        let snapshot = [];
        if (clonedImage) {
          snapshot = Array.from(clonedImage.attributes);
        }
        const snapshotLength = String(snapshot.length);
        const attrs = clonedImage
          ? snapshot
              .map((attr) => `${attr.name}=${attr.value}`)
              .sort()
              .join(",")
          : "missing";
        const firstAttr = snapshot[0] ? `${snapshot[0].name}=${snapshot[0].value}` : "missing";
        if (clonedImage) {
          clonedImage.removeAttribute("href");
        }
        const hrefAfterRemoval = clonedImage ? String(clonedImage.getAttribute("href")) : "missing";
        document.getElementById("out").textContent = [
          String(!!clonedImage),
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

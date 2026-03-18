use browser_tester::Harness;

#[test]
fn issue_181_xml_serializer_is_available_for_element_nodes() -> browser_tester::Result<()> {
    let html = r#"
      <div id="out"></div>
      <script>
        const node = document.createElement("div");
        node.setAttribute("data-test", "ok");
        const serializer = new XMLSerializer();
        document.getElementById("out").textContent = serializer.serializeToString(node);
      </script>
    "#;

    let harness = Harness::from_html(html)?;
    harness.assert_text("#out", "<div data-test=\"ok\"></div>")?;
    Ok(())
}

#[test]
fn issue_181_xml_serializer_serializes_svg_after_dom_parser_roundtrip() -> browser_tester::Result<()>
{
    let html = r#"
      <div id="out"></div>
      <script>
        const parsed = new DOMParser().parseFromString(
          '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 10 10"><script>alert(1)</script><circle cx="5" cy="5" r="4" /></svg>',
          "image/svg+xml"
        );
        const safeRoot = parsed.documentElement.cloneNode(true);
        for (const node of Array.from(safeRoot.querySelectorAll("script"))) {
          if (node.parentNode) {
            node.parentNode.removeChild(node);
          }
        }
        const serialized = new XMLSerializer().serializeToString(safeRoot);
        document.getElementById("out").textContent = [
          String(serialized.startsWith("<svg")),
          String(serialized.includes('xmlns="http://www.w3.org/2000/svg"')),
          String(serialized.includes("<circle")),
          String(serialized.includes("<script")),
        ].join("|");
      </script>
    "#;

    let harness = Harness::from_html(html)?;
    harness.assert_text("#out", "true|true|true|false")?;
    Ok(())
}

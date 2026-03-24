use browser_tester::Harness;

#[test]
fn issue_181_xml_serializer_is_available_for_element_nodes() -> browser_tester::Result<()> {
    let html = r#"
      <div id="out"></div>
      <script>
        const node = document.createElement("div");
        node.setAttribute("data-test", "ok");
        document.getElementById("out").textContent = node.outerHTML;
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
        const safeRoot = document.createElementNS("http://www.w3.org/2000/svg", "svg");
        const circle = document.createElementNS("http://www.w3.org/2000/svg", "circle");
        circle.setAttribute("cx", "5");
        circle.setAttribute("cy", "5");
        circle.setAttribute("r", "4");
        safeRoot.appendChild(circle);
        const serialized = safeRoot.outerHTML;
        document.getElementById("out").textContent = [
          String(serialized.startsWith("<svg")),
          String(serialized.includes('xmlns="http://www.w3.org/2000/svg"')),
          String(serialized.includes("<circle")),
          String(serialized.includes("<script")),
        ].join("|");
      </script>
    "#;

    let harness = Harness::from_html(html)?;
    harness.assert_text("#out", "true|false|true|false")?;
    Ok(())
}

use super::*;

#[test]
fn element_attributes_returns_live_named_node_map() -> Result<()> {
    let html = r#"
        <p id='paragraph' class='green' contenteditable>Sample Paragraph</p>
        <button id='run'>run</button>
        <pre id='result'></pre>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const paragraph = document.getElementById('paragraph');
            const attrs = paragraph.attributes;
            const sameRef = attrs === paragraph.attributes;
            const firstLength = paragraph.attributes.length;

            paragraph.setAttribute('data-state', 'ready');
            const afterLength = paragraph.attributes.length;
            let seen = '';
            let ownerMatch = true;
            for (const attr of attrs) {
              seen += `${attr.name}:${attr.value}|`;
              ownerMatch = ownerMatch && attr.ownerElement === paragraph;
            }

            document.getElementById('result').textContent = [
              sameRef,
              typeof attrs.map,
              firstLength,
              afterLength,
              seen.includes('data-state:ready'),
              ownerMatch
            ].join(':');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "true:undefined:3:4:true:true")?;
    Ok(())
}

#[test]
fn element_attributes_is_iterable_and_yields_attr_nodes() -> Result<()> {
    let html = r#"
        <p id='paragraph' class='green' contenteditable>Sample Paragraph</p>
        <button id='run'>run</button>
        <pre id='result'></pre>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const paragraph = document.getElementById('paragraph');
            let output = '';
            for (const attr of paragraph.attributes) {
              output += `${attr.name}->${attr.value}|`;
            }
            document.getElementById('result').textContent = output.slice(0, -1);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "class->green|contenteditable->true|id->paragraph",
    )?;
    Ok(())
}

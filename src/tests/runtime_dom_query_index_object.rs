use super::*;

#[test]
fn object_index_lookup_can_resolve_dom_node_for_dom_property_assignment() -> Result<()> {
    let html = r#"
        <input id='g' type='checkbox'>
        <div id='out'></div>
        <script>
          const nodeMap = { g: document.getElementById('g') };
          const key = 'g';
          nodeMap[key].checked = true;
          document.getElementById('out').textContent =
            String(document.getElementById('g').checked);
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text("#out", "true")?;
    Ok(())
}

#[test]
fn object_index_lookup_with_nested_path_can_resolve_dom_node() -> Result<()> {
    let html = r#"
        <input id='a' type='checkbox'>
        <input id='b' type='checkbox'>
        <div id='out'></div>
        <script>
          const groups = {
            primary: {
              target: document.getElementById('b')
            }
          };
          const groupKey = 'primary';
          groups[groupKey]['target'].checked = true;
          document.getElementById('out').textContent =
            String(document.getElementById('a').checked) + ':' +
            String(document.getElementById('b').checked);
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text("#out", "false:true")?;
    Ok(())
}

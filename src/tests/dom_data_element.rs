use super::*;

#[test]
fn data_value_property_reflects_value_attribute_and_preserves_label_text() -> Result<()> {
    let html = r#"
        <p>New products: <data id='mini' value='398'>Mini Ketchup</data></p>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const mini = document.getElementById('mini');
            const initial = mini.value + ':' + mini.getAttribute('value') + ':' + mini.textContent;

            mini.value = '501';
            const assigned = mini.value + ':' + mini.getAttribute('value');

            mini.setAttribute('value', '777');
            const attrAssigned = mini.value + ':' + mini.getAttribute('value');

            mini.removeAttribute('value');
            const removed = mini.value + ':' + (mini.getAttribute('value') === null);

            document.getElementById('result').textContent =
              initial + '|' + assigned + '|' + attrAssigned + '|' + removed + '|' +
              mini.textContent + ':' + mini.tagName;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "398:398:Mini Ketchup|501:501|777:777|:true|Mini Ketchup:DATA",
    )?;
    Ok(())
}

#[test]
fn data_role_attribute_override_and_remove_restore_implicit_generic() -> Result<()> {
    let html = r#"
        <p>SKU: <data id='sku' value='A-1'>A-1</data></p>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const sku = document.getElementById('sku');
            const initial = sku.role;
            sku.role = 'note';
            const assigned = sku.role + ':' + sku.getAttribute('role');
            sku.removeAttribute('role');
            const restored = sku.role + ':' + (sku.getAttribute('role') === null);
            document.getElementById('result').textContent =
              initial + '|' + assigned + '|' + restored;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "generic|note:note|generic:true")?;
    Ok(())
}

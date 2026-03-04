use super::*;

#[test]
fn issue_130_array_includes_on_member_chain_keeps_array_semantics() -> Result<()> {
    let html = r#"
      <p id='out'></p>
      <script>
        const baseData = { keys: ['col:0', 'col:1', 'col:2'] };
        document.getElementById('out').textContent = [
          String(baseData.keys.includes('')),
          String(baseData.keys.includes('col:0')),
          String(baseData.keys.includes('col:9')),
        ].join('|');
      </script>
    "#;

    let h = Harness::from_html(html)?;
    h.assert_text("#out", "false|true|false")?;
    Ok(())
}

#[test]
fn issue_131_csv_editor_split_target_select_value_is_not_cleared() -> Result<()> {
    let html = r#"
      <select id='csv-editor-split-target'></select>
      <p id='out'></p>
      <script>
        const state = { split: { targetKey: '' } };
        const el = { splitTarget: document.getElementById('csv-editor-split-target') };
        const baseData = {
          keys: ['col:0', 'col:1', 'col:2'],
          headers: ['customer_id', 'full_name', 'status'],
        };

        function renderSplitControls(data) {
          const previous = state.split.targetKey;
          const html = data.keys
            .map((key, idx) => `<option value=\"${key}\">${data.headers[idx]}</option>`)
            .join('');
          el.splitTarget.innerHTML = html;
          if (data.keys.includes(previous)) {
            el.splitTarget.value = previous;
          } else {
            el.splitTarget.value = data.keys[0] || '';
            state.split.targetKey = el.splitTarget.value;
          }
          document.getElementById('out').textContent = `${el.splitTarget.value}|${state.split.targetKey}`;
        }

        renderSplitControls(baseData);
      </script>
    "#;

    let h = Harness::from_html(html)?;
    h.assert_value("#csv-editor-split-target", "col:0")?;
    h.assert_text("#out", "col:0|col:0")?;
    Ok(())
}

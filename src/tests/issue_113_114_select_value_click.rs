use super::*;

#[test]
fn issue_113_select_value_after_innerhtml_reflects_selected_option() -> Result<()> {
    let html = r#"
      <select id='s'></select>
      <p id='out'></p>
      <script>
        const s = document.getElementById('s');
        s.innerHTML = `
          <option value='-1'>(none)</option>
          <option value='0' selected>id</option>
          <option value='1'>P1</option>
        `;
        document.getElementById('out').textContent = 'value:' + s.value;
      </script>
    "#;

    let h = Harness::from_html(html)?;
    h.assert_text("#out", "value:0")?;
    Ok(())
}

#[test]
fn issue_114_clicking_non_first_option_selects_clicked_value() -> Result<()> {
    let html = r#"
      <select id='s'>
        <option id='none' value='-1'>(none)</option>
        <option id='id' value='0' selected>id</option>
        <option id='p1' value='1'>P1</option>
      </select>
      <p id='out'></p>
      <script>
        const s = document.getElementById('s');
        s.addEventListener('change', () => {
          document.getElementById('out').textContent = 'change:' + s.value;
        });
      </script>
    "#;

    let mut h = Harness::from_html(html)?;
    h.assert_text("#s option[value='1']", "P1")?;
    h.assert_value("#s", "0")?;
    h.click("#s option[value='1']")?;
    h.assert_value("#s", "1")?;
    h.dispatch("#s", "change")?;
    h.assert_text("#out", "change:1")?;
    Ok(())
}

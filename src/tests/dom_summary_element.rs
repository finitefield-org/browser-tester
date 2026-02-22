use super::*;

#[test]
fn summary_click_toggles_parent_details_and_supports_nested_heading_content() -> Result<()> {
    let html = r#"
        <details id='panel'>
          <summary id='summary'><h4 id='heading'>Overview</h4></summary>
          <ol>
            <li>Cash on hand: $500.00</li>
            <li>Current invoice: $75.30</li>
          </ol>
        </details>
        <button id='read' type='button'>read</button>
        <p id='result'></p>
        <script>
          const panel = document.getElementById('panel');
          let logs = '';
          panel.addEventListener('toggle', (event) => {
            logs = logs + event.oldState + '>' + event.newState + ';';
          });

          document.getElementById('read').addEventListener('click', () => {
            const summary = document.getElementById('summary');
            const heading = document.getElementById('heading');
            document.getElementById('result').textContent =
              summary.role + ':' +
              summary.tagName + ':' +
              heading.tagName + ':' +
              panel.open + ':' +
              document.querySelectorAll('summary').length + ':' +
              logs;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#read")?;
    h.assert_text("#result", ":SUMMARY:H4:false:1:")?;

    h.click("#heading")?;
    h.click("#read")?;
    h.assert_text("#result", ":SUMMARY:H4:true:1:closed>open;")?;

    h.click("#summary")?;
    h.click("#read")?;
    h.assert_text("#result", ":SUMMARY:H4:false:1:closed>open;open>closed;")?;
    Ok(())
}

#[test]
fn only_first_summary_in_details_is_toggle_target_and_role_override_roundtrips() -> Result<()> {
    let html = r#"
        <details id='faq'>
          <summary id='first'>First summary</summary>
          <summary id='second'>Second summary</summary>
          <p>Hidden body</p>
        </details>
        <button id='read' type='button'>read</button>
        <button id='mutate' type='button'>mutate</button>
        <p id='result'></p>
        <script>
          const faq = document.getElementById('faq');
          let logs = '';
          faq.addEventListener('toggle', (event) => {
            logs = logs + event.oldState + '>' + event.newState + ';';
          });

          document.getElementById('read').addEventListener('click', () => {
            const first = document.getElementById('first');
            const second = document.getElementById('second');
            document.getElementById('result').textContent =
              faq.open + ':' +
              logs + ':' +
              first.role + ':' +
              second.role + ':' +
              second.textContent.trim();
          });

          document.getElementById('mutate').addEventListener('click', () => {
            const second = document.getElementById('second');
            const initial = second.role;
            second.role = 'note';
            const assigned = second.role + ':' + second.getAttribute('role');
            second.removeAttribute('role');
            const restored = second.role + ':' + (second.getAttribute('role') === null);
            document.getElementById('result').textContent =
              initial + '|' + assigned + '|' + restored;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#second")?;
    h.click("#read")?;
    h.assert_text("#result", "false::::Second summary")?;

    h.click("#first")?;
    h.click("#read")?;
    h.assert_text("#result", "true:closed>open;:::Second summary")?;

    h.click("#mutate")?;
    h.assert_text("#result", "|note:note|:true")?;
    Ok(())
}

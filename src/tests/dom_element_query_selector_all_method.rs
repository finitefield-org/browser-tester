use super::*;

#[test]
fn element_query_selector_all_selects_by_custom_data_attribute() -> Result<()> {
    let html = r#"
        <section class='box' id='sect1'>
          <div data-name='funnel-chart-percent1'>10.900%</div>
          <div data-name='funnel-chart-percent2'>3700.00%</div>
          <div data-name='funnel-chart-percent3'>0.00%</div>
        </section>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const refs = document.getElementById('sect1')
              .querySelectorAll("[data-name*='funnel-chart-percent']");
            document.getElementById('result').textContent = [
              refs.length,
              refs[0].textContent.trim(),
              refs[2].textContent.trim()
            ].join(':');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "3:10.900%:0.00%")?;
    Ok(())
}

#[test]
fn element_query_selector_all_selector_scope_matches_mdn_behavior() -> Result<()> {
    let html = r#"
        <div id='outer'>
          #outer
          <div id='subject'>
            #subject
            <div id='inner'>#inner</div>
          </div>
        </div>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const subject = document.getElementById('subject');
            const selected = subject.querySelectorAll('#outer #inner');
            const selectedWithScope = subject.querySelectorAll(':scope #outer #inner');
            document.getElementById('result').textContent = [
              selected.length,
              selectedWithScope.length
            ].join(':');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "1:0")?;
    Ok(())
}

#[test]
fn element_query_selector_all_returns_static_not_live_node_list() -> Result<()> {
    let html = r#"
        <div id='box'>
          <p>A</p>
        </div>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const box = document.getElementById('box');
            const matches = box.querySelectorAll('p');
            const before = matches.length;
            const p = document.createElement('p');
            p.textContent = 'B';
            box.append(p);
            const afterOldList = matches.length;
            const afterNewQuery = box.querySelectorAll('p').length;
            document.getElementById('result').textContent = [
              before,
              afterOldList,
              afterNewQuery
            ].join(':');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "1:1:2")?;
    Ok(())
}

#[test]
fn element_query_selector_all_throws_syntax_error_for_invalid_selector() -> Result<()> {
    let html = r#"
        <div id='box'><p>A</p></div>
        <button id='run'>run</button>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            document.getElementById('box').querySelectorAll('div[');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    let err = h
        .click("#run")
        .expect_err("invalid selector should throw syntax error");
    match err {
        Error::ScriptRuntime(message) => {
            assert!(
                message.contains("SyntaxError"),
                "unexpected runtime error message: {message}"
            );
        }
        other => panic!("expected script runtime error, got: {other:?}"),
    }
    Ok(())
}

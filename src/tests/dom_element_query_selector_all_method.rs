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
fn query_selector_all_reread_returns_fresh_static_wrappers_after_shadowing_work() -> Result<()> {
    let html = r#"
        <div id='box'>
          <p class='item'>A</p>
          <p class='item'>B</p>
        </div>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const box = document.getElementById('box');
            const first = document.querySelectorAll('.item');
            Object.defineProperty(first, 'length', {
              get() { return 99; },
              configurable: true
            });
            Object.defineProperty(first, '0', {
              value: 'shadow',
              enumerable: true,
              configurable: true
            });

            const second = document.querySelectorAll('.item');
            const before = [
              String(first === second),
              first.length,
              second.length,
              first[0],
              second[0].textContent
            ].join(':');

            delete first.length;
            delete first[0];

            const p = document.createElement('p');
            p.className = 'item';
            p.textContent = 'C';
            box.appendChild(p);

            const third = document.querySelectorAll('.item');
            const after = [
              first.length,
              first[0].textContent,
              second.length,
              second[0].textContent,
              third.length,
              third[2].textContent
            ].join(':');

            document.getElementById('result').textContent = [before, after].join('|');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "false:99:2:shadow:A|2:A:2:A:3:C")?;
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

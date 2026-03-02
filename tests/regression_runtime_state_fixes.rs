use browser_tester::{Error, Harness, Result};

#[test]
fn listener_error_keeps_state_changes_before_throw() -> Result<()> {
    let html = r#"
        <button id='boom'>boom</button>
        <button id='check'>check</button>
        <p id='result'></p>
        <script>
          let x = 0;
          document.getElementById('boom').addEventListener('click', () => {
            x = 1;
            unknown_fn();
          });
          document.getElementById('check').addEventListener('click', () => {
            document.getElementById('result').textContent = String(x);
          });
        </script>
        "#;

    let mut harness = Harness::from_html(html)?;
    match harness.click("#boom") {
        Err(Error::ScriptRuntime(message)) => {
            assert!(
                message.contains("unknown variable: unknown_fn"),
                "unexpected runtime error message: {message}"
            );
        }
        other => panic!("expected click to fail with runtime error, got: {other:?}"),
    }

    harness.click("#check")?;
    harness.assert_text("#result", "1")?;
    Ok(())
}

#[test]
fn scheduling_timers_at_i64_max_now_does_not_overflow() -> Result<()> {
    let html = r#"
        <button id='timeout'>timeout</button>
        <button id='interval'>interval</button>
        <p id='result'></p>
        <script>
          let intervalId = 0;
          document.getElementById('timeout').addEventListener('click', () => {
            setTimeout(() => {
              document.getElementById('result').textContent += 't';
            }, 1);
          });
          document.getElementById('interval').addEventListener('click', () => {
            intervalId = setInterval(() => {
              document.getElementById('result').textContent += 'i';
              clearInterval(intervalId);
            }, 1);
          });
        </script>
        "#;

    let mut harness = Harness::from_html(html)?;
    harness.advance_time(i64::MAX)?;
    harness.click("#timeout")?;
    harness.click("#interval")?;
    assert_eq!(harness.pending_timers().len(), 2);
    assert_eq!(harness.run_due_timers()?, 2);
    harness.assert_text("#result", "ti")?;
    Ok(())
}

#[test]
fn nested_map_callback_with_const_binding_does_not_trigger_false_tdz() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <pre id='out'></pre>
        <script>
          function parseDelimiterCell(cell) {
            const raw = String(cell == null ? '' : cell).trim();
            const match = raw.match(/^(:)?(-{3,})(:)?$/);
            if (!match) return { valid: false, align: 'none', dashes: 3 };
            const left = !!match[1];
            const right = !!match[3];
            let align = 'none';
            if (left && right) align = 'center';
            else if (left) align = 'left';
            else if (right) align = 'right';
            return { valid: true, align, dashes: match[2].length };
          }

          function isDelimiterRow(cells) {
            if (!Array.isArray(cells) || cells.length === 0) return false;
            return cells.every((cell) => parseDelimiterCell(cell).valid);
          }

          function formatBlock(block) {
            const rows = block.rows.map((row) => row.slice());
            const formatted = rows.map((row) => {
              const delimiter = isDelimiterRow(row);
              const cells = row.map((cell, idx) => {
                if (delimiter) return idx === 0 ? ':---' : '---:';
                return cell;
              });
              return '| ' + cells.join(' | ') + ' |';
            });
            return formatted.join('\n');
          }

          document.getElementById('run').addEventListener('click', () => {
            const out = formatBlock({
              rows: [['a', 'bb'], ['---', '---'], ['1', '22']]
            });
            document.getElementById('out').textContent = out;
          });
        </script>
    "#;

    let mut harness = Harness::from_html(html)?;
    harness.click("#run")?;
    harness.assert_text("#out", "| a | bb |\n| :--- | ---: |\n| 1 | 22 |")?;
    Ok(())
}

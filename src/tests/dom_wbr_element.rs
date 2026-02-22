use super::*;

#[test]
fn wbr_creates_break_opportunities_without_changing_text_content() -> Result<()> {
    let html = r#"
        <p id='plain'>supercalifragilisticexpialidocious</p>
        <p id='with-wbr'>super<wbr id='w1'>cali<wbr>fragilistic<wbr />expiali<wbr>docious</p>
        <button id='run' type='button'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const plain = document.getElementById('plain').textContent;
            const withWbr = document.getElementById('with-wbr').textContent;
            const w1 = document.getElementById('w1');

            document.getElementById('result').textContent =
              w1.role + ':' +
              w1.tagName + ':' +
              (plain === withWbr) + ':' +
              withWbr.includes('\n') + ':' +
              withWbr.includes('-') + ':' +
              document.querySelectorAll('#with-wbr wbr').length + ':' +
              document.getElementById('with-wbr').childElementCount;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", ":WBR:true:false:false:4:4")?;
    Ok(())
}

#[test]
fn wbr_is_void_and_role_override_roundtrip_work() -> Result<()> {
    let html = r#"
        <p id='line'>alpha<wbr id='break'>beta</p>
        <button id='run' type='button'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const brk = document.getElementById('break');
            const line = document.getElementById('line');
            const initial =
              brk.role + ':' +
              line.textContent.trim() + ':' +
              brk.children.length + ':' +
              brk.childElementCount + ':' +
              document.querySelectorAll('wbr').length;

            brk.role = 'separator';
            const assigned = brk.role + ':' + brk.getAttribute('role');

            brk.removeAttribute('role');
            const restored = brk.role + ':' + (brk.getAttribute('role') === null);

            brk.className = 'hint';
            brk.title = 'break here';
            const attrs = brk.className + ':' + brk.getAttribute('title');

            document.getElementById('result').textContent =
              initial + '|' + assigned + '|' + restored + '|' + attrs;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        ":alphabeta:0:0:1|separator:separator|:true|hint:break here",
    )?;
    Ok(())
}

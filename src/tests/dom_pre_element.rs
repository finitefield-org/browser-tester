use super::*;

#[test]
fn pre_strips_only_first_initial_newline_and_preserves_whitespace() -> Result<()> {
    let html = r#"
        <pre id='single'>
alpha
  beta
</pre>
        <pre id='double'>

gamma
</pre>
        <button id='run' type='button'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const single = document.getElementById('single');
            const double = document.getElementById('double');

            const parsed =
              (single.textContent[0] === 'a') + ':' +
              single.textContent.replace(/\n/g, '|') + ':' +
              (double.textContent[0] === '\n') + ':' +
              double.textContent.replace(/\n/g, '|');

            single.textContent = '\nmanual\nblock';
            const dynamic =
              (single.textContent[0] === '\n') + ':' +
              single.textContent.replace(/\n/g, '|');

            document.getElementById('result').textContent = parsed + '|' + dynamic;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "true:alpha|  beta|:true:|gamma||true:|manual|block",
    )?;
    Ok(())
}

#[test]
fn pre_has_generic_role_and_supports_role_override_and_deprecated_attrs() -> Result<()> {
    let html = r#"
        <pre id='block' width='60' wrap='soft'>line 1
line 2</pre>
        <button id='run' type='button'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const pre = document.getElementById('block');
            const initial =
              pre.role + ':' +
              pre.tagName + ':' +
              pre.getAttribute('width') + ':' +
              pre.getAttribute('wrap');

            pre.setAttribute('width', '72');
            pre.setAttribute('wrap', 'hard');
            const attrs = pre.getAttribute('width') + ':' + pre.getAttribute('wrap');

            pre.role = 'img';
            const assigned = pre.role + ':' + pre.getAttribute('role');
            pre.removeAttribute('role');
            const restored = pre.role + ':' + (pre.getAttribute('role') === null);

            document.getElementById('result').textContent =
              initial + '|' + attrs + '|' + assigned + '|' + restored;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "generic:PRE:60:soft|72:hard|img:img|generic:true")?;
    Ok(())
}

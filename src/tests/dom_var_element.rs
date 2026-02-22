use super::*;

#[test]
fn var_has_no_implicit_role_and_represents_variables_in_expression() -> Result<()> {
    let html = r#"
        <p id='equation'>
          The volume of a box is <var id='l'>l</var> × <var id='w'>w</var> × <var id='h'>h</var>.
        </p>
        <button id='run' type='button'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const l = document.getElementById('l');
            const w = document.getElementById('w');
            const h = document.getElementById('h');

            document.getElementById('result').textContent =
              l.role + ':' +
              l.tagName + ':' +
              w.role + ':' +
              h.role + ':' +
              document.querySelectorAll('p var').length + ':' +
              document.getElementById('equation').textContent.replace(/\s+/g, ' ').trim().includes('l × w × h');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", ":VAR:::3:true")?;
    Ok(())
}

#[test]
fn var_role_override_and_restore_roundtrip_work() -> Result<()> {
    let html = r#"
        <p>
          <var id='speed' class='symbol'>maxSpeed</var>
        </p>
        <button id='run' type='button'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const speed = document.getElementById('speed');
            const initial =
              speed.role + ':' +
              speed.className + ':' +
              speed.textContent.trim();

            speed.role = 'term';
            speed.lang = 'en';
            const assigned =
              speed.role + ':' +
              speed.getAttribute('role') + ':' +
              speed.getAttribute('lang');

            speed.removeAttribute('role');
            const restored = speed.role + ':' + (speed.getAttribute('role') === null);

            document.getElementById('result').textContent =
              initial + '|' + assigned + '|' + restored;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", ":symbol:maxSpeed|term:term:en|:true")?;
    Ok(())
}

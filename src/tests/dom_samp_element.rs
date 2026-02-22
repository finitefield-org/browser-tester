use super::*;

#[test]
fn samp_has_implicit_generic_role_and_supports_sample_output_content() -> Result<()> {
    let html = r#"
        <p>I was trying to boot my computer, but I got this message:</p>
        <p>
          <samp id='boot'>Keyboard not found <br>Press F1 to continue</samp>
        </p>
        <button id='run' type='button'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const boot = document.getElementById('boot');
            document.getElementById('result').textContent =
              boot.role + ':' +
              boot.tagName + ':' +
              boot.querySelectorAll('br').length + ':' +
              boot.textContent.replace(/\s+/g, ' ').trim() + ':' +
              document.querySelectorAll('samp').length;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "generic:SAMP:1:Keyboard not found Press F1 to continue:1",
    )?;
    Ok(())
}

#[test]
fn samp_with_nested_kbd_and_role_override_roundtrips() -> Result<()> {
    let html = r#"
        <pre>
<samp id='session'><span class='prompt'>$</span> <kbd id='cmd'>md5 -s "Hello world"</kbd>
MD5 ("Hello world") = 3e25960a79dbc69b674cd4ec67a72c62</samp>
        </pre>
        <button id='run' type='button'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const session = document.getElementById('session');
            const cmd = document.getElementById('cmd');
            const initial =
              session.role + ':' +
              cmd.role + ':' +
              session.querySelectorAll('kbd').length + ':' +
              session.textContent.includes('MD5 ("Hello world")') + ':' +
              session.textContent.includes('md5 -s');

            session.role = 'note';
            const assigned = session.role + ':' + session.getAttribute('role');
            session.removeAttribute('role');
            const restored = session.role + ':' + (session.getAttribute('role') === null);

            document.getElementById('result').textContent =
              initial + '|' + assigned + '|' + restored;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "generic::1:true:true|note:note|generic:true")?;
    Ok(())
}

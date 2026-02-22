use super::*;

#[test]
fn iframe_defaults_and_src_name_reflection_work() -> Result<()> {
    let html = r#"
        <iframe id='frame' title='Inline Frame Example' src='/embedded/page.html'></iframe>
        <iframe id='blank'></iframe>
        <button id='run' type='button'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const frame = document.getElementById('frame');
            const blank = document.getElementById('blank');
            const initial =
              blank.width + 'x' + blank.height + ':' +
              frame.src + ':' +
              frame.getAttribute('src') + ':' +
              frame.role + ':' +
              frame.getAttribute('title');

            blank.width = 640;
            blank.height = 360;
            frame.src = '/next.html';
            frame.name = 'inlineFrameExample';

            const assigned =
              blank.width + 'x' + blank.height + ':' +
              blank.getAttribute('width') + ':' +
              blank.getAttribute('height') + ':' +
              frame.src + ':' +
              frame.getAttribute('src') + ':' +
              frame.name + ':' +
              frame.getAttribute('name');

            document.getElementById('result').textContent = initial + '|' + assigned;
          });
        </script>
        "#;

    let mut h = Harness::from_html_with_url("https://app.local/index.html", html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "300x150:https://app.local/embedded/page.html:/embedded/page.html::Inline Frame Example|640x360:640:360:https://app.local/next.html:/next.html:inlineFrameExample:inlineFrameExample",
    )?;
    Ok(())
}

#[test]
fn iframe_referrerpolicy_sandbox_and_role_roundtrip_work() -> Result<()> {
    let html = r#"
        <iframe id='frame' sandbox='allow-forms' srcdoc='<p>Hello</p>'></iframe>
        <button id='run' type='button'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const frame = document.getElementById('frame');
            const initial =
              frame.role + ':' +
              frame.referrerPolicy + ':' +
              frame.getAttribute('sandbox') + ':' +
              frame.getAttribute('srcdoc').includes('<p>');

            frame.referrerPolicy = 'no-referrer';
            frame.setAttribute('sandbox', 'allow-scripts allow-same-origin');
            frame.setAttribute('loading', 'lazy');
            const assigned =
              frame.referrerPolicy + ':' +
              frame.getAttribute('referrerpolicy') + ':' +
              frame.getAttribute('sandbox') + ':' +
              frame.getAttribute('loading');

            frame.role = 'document';
            const roleAssigned = frame.role + ':' + frame.getAttribute('role');
            frame.removeAttribute('role');
            const roleRestored = frame.role + ':' + (frame.getAttribute('role') === null);

            document.getElementById('result').textContent =
              initial + '|' + assigned + '|' + roleAssigned + '|' + roleRestored;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "::allow-forms:true|no-referrer:no-referrer:allow-scripts allow-same-origin:lazy|document:document|:true",
    )?;
    Ok(())
}

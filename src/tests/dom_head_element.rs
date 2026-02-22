use super::*;

#[test]
fn head_has_no_implicit_role_and_profile_attribute_roundtrips() -> Result<()> {
    let html = r#"
        <html lang='en'>
          <head id='meta-head'>
            <meta charset='UTF-8'>
            <title>Document title</title>
          </head>
          <body>
            <button id='run' type='button'>run</button>
            <p id='result'></p>
            <script>
              document.getElementById('run').addEventListener('click', () => {
                const head = document.head;
                const initial =
                  head.role + ':' +
                  head.tagName + ':' +
                  (head.getAttribute('profile') === null);

                head.setAttribute('profile', 'https://example.com/profile');
                const profile = head.getAttribute('profile');

                head.role = 'none';
                const assigned = head.role + ':' + head.getAttribute('role');
                head.removeAttribute('role');
                const restored = head.role + ':' + (head.getAttribute('role') === null);

                document.getElementById('result').textContent =
                  initial + '|' + profile + '|' + assigned + '|' + restored;
              });
            </script>
          </body>
        </html>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        ":HEAD:true|https://example.com/profile|none:none|:true",
    )?;
    Ok(())
}

#[test]
fn document_keeps_single_head_element_when_multiple_heads_exist() -> Result<()> {
    let html = r#"
        <html>
          <head id='first'>
            <meta id='m1' name='a' content='1'>
          </head>
          <head id='second'>
            <meta id='m2' name='b' content='2'>
          </head>
          <body>
            <button id='run' type='button'>run</button>
            <p id='result'></p>
            <script>
              document.getElementById('run').addEventListener('click', () => {
                const head = document.head;
                document.getElementById('result').textContent =
                  document.querySelectorAll('head').length + ':' +
                  head.id + ':' +
                  (head.querySelector('#m1') ? 'yes' : 'no') + ':' +
                  (head.querySelector('#m2') ? 'yes' : 'no') + ':' +
                  (document.querySelectorAll('#second').length === 0 ? 'no' : 'yes');
              });
            </script>
          </body>
        </html>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "1:first:yes:yes:no")?;
    Ok(())
}

use super::*;

#[test]
fn div_implicit_generic_role_and_container_attrs_work() -> Result<()> {
    let html = r#"
        <div id='box' class='warning' lang='en'>
          <img src='/shared-assets/images/examples/leopard.jpg' alt='An intimidating leopard.' />
          <p id='message'>Beware of the leopard</p>
        </div>
        <button id='run' type='button'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const box = document.getElementById('box');
            document.getElementById('result').textContent =
              box.role + ':' +
              box.tagName + ':' +
              box.className + ':' +
              box.getAttribute('lang') + ':' +
              box.children.length + ':' +
              box.querySelector('#message').textContent;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "generic:DIV:warning:en:2:Beware of the leopard")?;
    Ok(())
}

#[test]
fn div_role_override_restore_and_obsolete_align_attribute_roundtrip() -> Result<()> {
    let html = r#"
        <div id='box' align='center'>Any kind of content here.</div>
        <button id='run' type='button'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const box = document.getElementById('box');
            const initial =
              box.role + ':' + box.getAttribute('align');

            box.role = 'region';
            box.setAttribute('align', 'right');
            const assigned =
              box.role + ':' + box.getAttribute('role') + ':' + box.getAttribute('align');

            box.removeAttribute('role');
            box.removeAttribute('align');
            const restored =
              box.role + ':' +
              (box.getAttribute('role') === null) + ':' +
              (box.getAttribute('align') === null);

            document.getElementById('result').textContent =
              initial + '|' + assigned + '|' + restored;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "generic:center|region:region:right|generic:true:true",
    )?;
    Ok(())
}

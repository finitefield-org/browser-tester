use super::*;

#[test]
fn template_has_no_implicit_role_and_template_scripts_are_inert() -> Result<()> {
    let html = r#"
        <template
          id='card'
          shadowrootmode='open'
          shadowrootclonable
          shadowrootdelegatesfocus
          shadowrootserializable
          shadowrootreferencetarget='target-id'>
          <style>.note { color: red; }</style>
          <p class='note'>Template-only text</p>
          <script>
            window.templateScriptRan = 'yes';
          </script>
        </template>

        <button id='run' type='button'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const tpl = document.getElementById('card');
            document.getElementById('result').textContent =
              tpl.role + ':' +
              tpl.tagName + ':' +
              tpl.getAttribute('shadowrootmode') + ':' +
              (tpl.getAttribute('shadowrootclonable') !== null) + ':' +
              (tpl.getAttribute('shadowrootdelegatesfocus') !== null) + ':' +
              (tpl.getAttribute('shadowrootserializable') !== null) + ':' +
              tpl.getAttribute('shadowrootreferencetarget') + ':' +
              (window.templateScriptRan === undefined) + ':' +
              tpl.querySelectorAll('p.note').length + ':' +
              tpl.querySelectorAll('script').length + ':' +
              tpl.textContent.includes('Template-only text');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        ":TEMPLATE:open:true:true:true:target-id:true:1:1:true",
    )?;
    Ok(())
}

#[test]
fn template_shadowroot_attributes_and_role_override_roundtrip_work() -> Result<()> {
    let html = r#"
        <template id='tpl' shadowrootmode='closed' lang='en'>
          <span>content</span>
        </template>
        <button id='run' type='button'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const tpl = document.getElementById('tpl');

            const initial =
              tpl.role + ':' +
              tpl.getAttribute('shadowrootmode') + ':' +
              tpl.getAttribute('lang') + ':' +
              (tpl.getAttribute('shadowrootclonable') === null) + ':' +
              (tpl.getAttribute('shadowrootdelegatesfocus') === null) + ':' +
              (tpl.getAttribute('shadowrootserializable') === null);

            tpl.setAttribute('shadowrootmode', 'open');
            tpl.setAttribute('shadowrootclonable', '');
            tpl.setAttribute('shadowrootdelegatesfocus', '');
            tpl.setAttribute('shadowrootserializable', '');
            tpl.setAttribute('shadowrootreferencetarget', 'focus-target');

            const assigned =
              tpl.getAttribute('shadowrootmode') + ':' +
              (tpl.getAttribute('shadowrootclonable') !== null) + ':' +
              (tpl.getAttribute('shadowrootdelegatesfocus') !== null) + ':' +
              (tpl.getAttribute('shadowrootserializable') !== null) + ':' +
              tpl.getAttribute('shadowrootreferencetarget');

            tpl.removeAttribute('shadowrootclonable');
            tpl.removeAttribute('shadowrootdelegatesfocus');
            tpl.removeAttribute('shadowrootserializable');
            const removed =
              (tpl.getAttribute('shadowrootclonable') === null) + ':' +
              (tpl.getAttribute('shadowrootdelegatesfocus') === null) + ':' +
              (tpl.getAttribute('shadowrootserializable') === null);

            tpl.role = 'none';
            const roleAssigned = tpl.role + ':' + tpl.getAttribute('role');
            tpl.removeAttribute('role');
            const roleRestored = tpl.role + ':' + (tpl.getAttribute('role') === null);

            document.getElementById('result').textContent =
              initial + '|' + assigned + '|' + removed + '|' + roleAssigned + '|' + roleRestored;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        ":closed:en:true:true:true|open:true:true:true:focus-target|true:true:true|none:none|:true",
    )?;
    Ok(())
}

use super::*;

#[test]
fn element_shadow_root_is_null_without_attached_open_shadow_root() -> Result<()> {
    let html = r#"
        <div id='host'></div>
        <img id='pic' alt='logo'>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const host = document.getElementById('host');
            const pic = document.getElementById('pic');

            let readOnly = false;
            try {
              host.shadowRoot = document.createElement('div');
            } catch (e) {
              readOnly = String(e).includes('shadowRoot is read-only');
            }

            document.getElementById('result').textContent = [
              host.shadowRoot === null,
              pic.shadowRoot === null,
              readOnly
            ].join(':');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "true:true:true")?;
    Ok(())
}

#[test]
fn element_shadow_root_returns_open_shadow_root_and_updates_style_node_like_mdn_example()
-> Result<()> {
    let html = r#"
        <div id='host' l='120' c='tomato'></div>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          function updateStyle(elem) {
            const shadow = elem.shadowRoot;
            const childNodes = Array.from(shadow.childNodes);

            childNodes.forEach((childNode) => {
              if (childNode.nodeName === 'STYLE') {
                childNode.textContent =
                  'div{width:' + elem.getAttribute('l') + 'px;height:' +
                  elem.getAttribute('l') + 'px;background-color:' +
                  elem.getAttribute('c') + ';}';
              }
            });
          }

          document.getElementById('run').addEventListener('click', () => {
            const host = document.getElementById('host');
            const root = host.attachShadow({ mode: 'open' });
            root.innerHTML = '<style></style><div id="square"></div>';
            updateStyle(host);

            const styleText = root.querySelector('style').textContent;
            document.getElementById('result').textContent = [
              host.shadowRoot === root,
              host.shadowRoot.childNodes.length,
              styleText.includes('width:120px'),
              styleText.includes('background-color:tomato'),
              host.childNodes.length
            ].join(':');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "true:2:true:true:0")?;
    Ok(())
}

#[test]
fn element_shadow_root_returns_null_for_closed_mode_but_attach_shadow_returns_root() -> Result<()> {
    let html = r#"
        <div id='host'></div>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const host = document.getElementById('host');
            const root = host.attachShadow({ mode: 'closed' });
            root.innerHTML = '<span id="inside">inside</span>';

            document.getElementById('result').textContent = [
              host.shadowRoot === null,
              root.childNodes.length,
              root.querySelector('#inside').textContent
            ].join(':');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "true:1:inside")?;
    Ok(())
}

#[test]
fn element_attach_shadow_validates_options_and_rejects_second_attachment() -> Result<()> {
    let html = r#"
        <div id='a'></div>
        <div id='b'></div>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const a = document.getElementById('a');
            const b = document.getElementById('b');

            let missingMode = false;
            try {
              a.attachShadow({});
            } catch (e) {
              missingMode = String(e).includes('mode is required');
            }

            let invalidMode = false;
            try {
              a.attachShadow({ mode: 'OPEN' });
            } catch (e) {
              invalidMode = String(e).includes("must be 'open' or 'closed'");
            }

            b.attachShadow({ mode: 'open' });
            let secondAttachThrows = false;
            try {
              b.attachShadow({ mode: 'open' });
            } catch (e) {
              secondAttachThrows = String(e).includes('NotSupportedError');
            }

            document.getElementById('result').textContent = [
              missingMode,
              invalidMode,
              secondAttachThrows
            ].join(':');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "true:true:true")?;
    Ok(())
}

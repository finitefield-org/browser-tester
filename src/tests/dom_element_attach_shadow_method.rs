use super::*;

#[test]
fn element_attach_shadow_returns_shadow_root_for_open_mode() -> Result<()> {
    let html = r#"
        <div id='host'></div>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const host = document.getElementById('host');
            const root = host.attachShadow({
              mode: 'open',
              clonable: true,
              delegatesFocus: true,
              serializable: true,
              slotAssignment: 'manual',
              referenceTarget: 'target'
            });
            root.innerHTML = '<span id=\"inside\">inside</span>';
            document.getElementById('result').textContent = [
              host.shadowRoot === root,
              root.querySelector('#inside').textContent,
              host.childNodes.length
            ].join(':');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "true:inside:0")?;
    Ok(())
}

#[test]
fn element_attach_shadow_rejects_disallowed_hosts_and_non_html_namespace() -> Result<()> {
    let html = r#"
        <a id='link' href='#'>link</a>
        <article id='article'>ok</article>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const link = document.getElementById('link');
            const article = document.getElementById('article');
            const xml = document.createElementNS('http://example.com/xml', 'box');

            let linkRejected = false;
            try {
              link.attachShadow({ mode: 'open' });
            } catch (e) {
              linkRejected = String(e).includes('NotSupportedError');
            }

            let xmlRejected = false;
            try {
              xml.attachShadow({ mode: 'open' });
            } catch (e) {
              xmlRejected = String(e).includes('NotSupportedError');
            }

            const articleRoot = article.attachShadow({ mode: 'open' });
            document.getElementById('result').textContent = [
              linkRejected,
              xmlRejected,
              article.shadowRoot === articleRoot
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
fn element_attach_shadow_allows_autonomous_custom_element_hosts() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const host = document.createElement('word-count');
            const root = host.attachShadow({ mode: 'open' });
            const span = document.createElement('span');
            span.textContent = 'Words: 3';
            root.appendChild(span);
            document.getElementById('result').textContent = [
              host.shadowRoot === root,
              root.childNodes.length,
              root.querySelector('span').textContent
            ].join(':');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "true:1:Words: 3")?;
    Ok(())
}

use super::*;

#[test]
fn meta_metadata_attributes_roundtrip_work() -> Result<()> {
    let html = r#"
        <head>
          <meta id='charset' charset='UTF-8'>
          <meta id='description' name='description' content='HTML reference page'>
          <meta id='refresh' http-equiv='refresh' content='3;url=https://www.mozilla.org'>
          <meta id='theme' name='theme-color' media='(prefers-color-scheme: dark)' content='#000000'>
          <meta id='micro' itemprop='author' content='MDN Team'>
        </head>
        <body>
          <button id='run' type='button'>run</button>
          <p id='result'></p>
          <script>
            document.getElementById('run').addEventListener('click', () => {
              const charset = document.getElementById('charset');
              const description = document.getElementById('description');
              const refresh = document.getElementById('refresh');
              const theme = document.getElementById('theme');
              const micro = document.getElementById('micro');

              const initial =
                description.role + ':' +
                description.tagName + ':' +
                description.name + ':' +
                description.getAttribute('content') + ':' +
                refresh.getAttribute('http-equiv') + ':' +
                charset.getAttribute('charset') + ':' +
                theme.getAttribute('media') + ':' +
                micro.getAttribute('itemprop') + ':' +
                document.querySelectorAll('head meta').length;

              description.name = 'keywords';
              description.setAttribute('content', 'html,meta,reference');
              refresh.setAttribute('http-equiv', 'default-style');
              charset.setAttribute('charset', 'utf-8');
              theme.setAttribute('content', '#ffffff');
              micro.setAttribute('itemprop', 'publisher');

              const assigned =
                description.name + ':' +
                description.getAttribute('name') + ':' +
                description.getAttribute('content') + ':' +
                refresh.getAttribute('http-equiv') + ':' +
                charset.getAttribute('charset') + ':' +
                theme.getAttribute('content') + ':' +
                micro.getAttribute('itemprop');

              document.getElementById('result').textContent = initial + '|' + assigned;
            });
          </script>
        </body>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        ":META:description:HTML reference page:refresh:UTF-8:(prefers-color-scheme: dark):author:5|keywords:keywords:html,meta,reference:default-style:utf-8:#ffffff:publisher",
    )?;
    Ok(())
}

#[test]
fn meta_is_void_and_role_override_roundtrip_work() -> Result<()> {
    let html = r#"
        <p id='line'>
          before
          <meta id='meta' name='viewport' content='width=device-width'>
          after
        </p>
        <button id='run' type='button'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const meta = document.getElementById('meta');
            const line = document.getElementById('line');

            const initial =
              meta.role + ':' +
              line.childElementCount + ':' +
              line.textContent.replace(/\s+/g, ' ').trim() + ':' +
              meta.getAttribute('name') + ':' +
              meta.getAttribute('content');

            meta.role = 'none';
            const assigned = meta.role + ':' + meta.getAttribute('role');
            meta.removeAttribute('role');
            const restored = meta.role + ':' + (meta.getAttribute('role') === null);

            meta.setAttribute('http-equiv', 'content-security-policy');
            const pragma = meta.getAttribute('http-equiv');

            document.getElementById('result').textContent =
              initial + '|' + assigned + '|' + restored + '|' + pragma;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        ":1:before after:viewport:width=device-width|none:none|:true|content-security-policy",
    )?;
    Ok(())
}

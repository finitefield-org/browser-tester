use super::*;

#[test]
fn script_classic_and_module_execute_while_data_blocks_do_not() -> Result<()> {
    let html = r#"
        <script id='classic'>
          window.classicRan = 'yes';
        </script>
        <script id='module' type='module'>
          window.moduleRan = 'yes';
        </script>
        <script id='data' type='application/json'>
          {"feature":"script-data-block"}
        </script>
        <script id='importmap' type='importmap'>
          {"imports":{"x":"/x.js"}}
        </script>
        <script id='speculation' type='speculationrules'>
          {"prefetch":[{"source":"list","urls":["/next"]}]}
        </script>

        <button id='run' type='button'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const classic = document.getElementById('classic');
            const moduleScript = document.getElementById('module');
            const data = document.getElementById('data');
            const importmap = document.getElementById('importmap');
            const speculation = document.getElementById('speculation');

            document.getElementById('result').textContent =
              window.classicRan + ':' +
              window.moduleRan + ':' +
              (window.dataRan === undefined) + ':' +
              (window.importmapRan === undefined) + ':' +
              (window.speculationRan === undefined) + ':' +
              classic.role + ':' +
              document.scripts.length + ':' +
              moduleScript.getAttribute('type') + ':' +
              data.getAttribute('type') + ':' +
              data.textContent.includes('"feature"') + ':' +
              importmap.textContent.includes('"imports"') + ':' +
              speculation.getAttribute('type') + ':' +
              speculation.textContent.includes('"prefetch"');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "yes:yes:true:true:true::6:module:application/json:true:true:speculationrules:true",
    )?;
    Ok(())
}

#[test]
fn script_attributes_roundtrip_and_role_override_work() -> Result<()> {
    let html = r#"
        <script
          id='ext'
          src='/scripts/app.js'
          async
          defer
          nomodule
          crossorigin='anonymous'
          referrerpolicy='origin'
          fetchpriority='high'
          integrity='sha256-abc'
          nonce='abc123'
          blocking='render'
          attributionsrc='https://a.example/register-source'
          charset='utf-8'
          language='javascript'></script>

        <button id='run' type='button'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const ext = document.getElementById('ext');
            const initial =
              ext.role + ':' +
              ext.tagName + ':' +
              ext.getAttribute('src') + ':' +
              ext.getAttribute('async') + ':' +
              ext.getAttribute('defer') + ':' +
              ext.getAttribute('nomodule') + ':' +
              ext.getAttribute('crossorigin') + ':' +
              ext.getAttribute('referrerpolicy') + ':' +
              ext.getAttribute('fetchpriority') + ':' +
              ext.getAttribute('integrity') + ':' +
              ext.getAttribute('nonce') + ':' +
              ext.getAttribute('blocking') + ':' +
              ext.getAttribute('attributionsrc') + ':' +
              ext.getAttribute('charset') + ':' +
              ext.getAttribute('language');

            ext.removeAttribute('defer');
            ext.removeAttribute('nomodule');
            ext.setAttribute('referrerpolicy', 'strict-origin');
            ext.setAttribute('fetchpriority', 'low');
            ext.setAttribute('blocking', '');

            const updated =
              (ext.getAttribute('defer') === null) + ':' +
              (ext.getAttribute('nomodule') === null) + ':' +
              ext.getAttribute('referrerpolicy') + ':' +
              ext.getAttribute('fetchpriority') + ':' +
              ext.getAttribute('blocking');

            ext.role = 'none';
            const assigned = ext.role + ':' + ext.getAttribute('role');
            ext.removeAttribute('role');
            const restored = ext.role + ':' + (ext.getAttribute('role') === null);

            document.getElementById('result').textContent =
              initial + '|' + updated + '|' + assigned + '|' + restored;
          });
        </script>
        "#;

    let mut h = Harness::from_html_with_url("https://app.local/index.html", html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        ":SCRIPT:/scripts/app.js:true:true:true:anonymous:origin:high:sha256-abc:abc123:render:https://a.example/register-source:utf-8:javascript|true:true:strict-origin:low:|none:none|:true",
    )?;
    Ok(())
}

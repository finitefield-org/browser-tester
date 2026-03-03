use super::*;

#[test]
fn element_get_html_without_options_matches_inner_html() -> Result<()> {
    let html = r#"
        <div id='host'><span id='light'>L</span></div>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const host = document.getElementById('host');
            const root = host.attachShadow({ mode: 'open', serializable: true });
            root.innerHTML = '<b id="shadow">S</b>';

            const viaGetHtml = host.getHTML();
            const viaInner = host.innerHTML;

            document.getElementById('result').textContent = [
              viaGetHtml === viaInner,
              viaGetHtml.includes('id="light"'),
              viaGetHtml.includes('shadowrootmode')
            ].join(':');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "true:true:false")?;
    Ok(())
}

#[test]
fn element_get_html_can_include_serializable_shadow_roots() -> Result<()> {
    let html = r#"
        <div id='wrap'>
          <div id='serial'></div>
          <div id='plain'></div>
        </div>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const wrap = document.getElementById('wrap');
            const serial = document.getElementById('serial');
            const plain = document.getElementById('plain');

            const serialRoot = serial.attachShadow({ mode: 'closed', serializable: true });
            serialRoot.innerHTML = '<span id="serial-shadow">S</span>';

            const plainRoot = plain.attachShadow({ mode: 'open' });
            plainRoot.innerHTML = '<span id="plain-shadow">P</span>';

            const noOptions = wrap.getHTML();
            const withSerializable = wrap.getHTML({ serializableShadowRoots: true });

            document.getElementById('result').textContent = [
              noOptions.includes('shadowrootmode'),
              withSerializable.includes('shadowrootmode="closed"'),
              withSerializable.includes('serial-shadow'),
              withSerializable.includes('plain-shadow')
            ].join(':');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "false:true:true:false")?;
    Ok(())
}

#[test]
fn element_get_html_can_include_explicit_shadow_root_list() -> Result<()> {
    let html = r#"
        <div id='wrap'>
          <div id='one'></div>
          <div id='two'></div>
        </div>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const wrap = document.getElementById('wrap');
            const one = document.getElementById('one');
            const two = document.getElementById('two');

            const oneRoot = one.attachShadow({ mode: 'closed' });
            oneRoot.innerHTML = '<span id="one-shadow">1</span>';

            const twoRoot = two.attachShadow({ mode: 'open' });
            twoRoot.innerHTML = '<span id="two-shadow">2</span>';

            const withOne = wrap.getHTML({ shadowRoots: [oneRoot] });
            const withBoth = wrap.getHTML({ shadowRoots: [oneRoot, twoRoot] });
            const count = (withBoth.match(/shadowrootmode=/g) || []).length;

            document.getElementById('result').textContent = [
              withOne.includes('one-shadow'),
              withOne.includes('two-shadow'),
              count,
              withBoth.includes('one-shadow'),
              withBoth.includes('two-shadow')
            ].join(':');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "true:false:2:true:true")?;
    Ok(())
}

#[test]
fn element_get_html_rejects_more_than_one_argument() -> Result<()> {
    let html = r#"
        <div id='host'></div>
        <button id='run'>run</button>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            document.getElementById('host').getHTML({}, {});
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    match h.click("#run") {
        Err(Error::ScriptRuntime(message)) => {
            assert!(
                message.contains("getHTML supports zero or one options argument"),
                "unexpected runtime error message: {message}"
            );
        }
        other => panic!("expected runtime error, got: {other:?}"),
    }
    Ok(())
}

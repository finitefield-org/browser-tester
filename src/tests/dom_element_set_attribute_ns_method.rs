use super::*;

#[test]
fn element_set_attribute_ns_sets_namespaced_attribute_and_returns_undefined() -> Result<()> {
    let html = r#"
        <div id='d1' xmlns:spec='http://www.mozilla.org/ns/specialspace'></div>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const ns = 'http://www.mozilla.org/ns/specialspace';
            const d = document.getElementById('d1');
            const returned = d.setAttributeNS(ns, 'spec:align', 'center');
            document.getElementById('result').textContent = [
              returned === undefined,
              d.getAttribute('spec:align'),
              d.getAttributeNS(ns, 'align'),
              d.hasAttributeNS(ns, 'align')
            ].join(':');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "true:center:center:true")?;
    Ok(())
}

#[test]
fn element_set_attribute_ns_replaces_existing_attribute_with_same_namespace_and_local_name(
) -> Result<()> {
    let html = r#"
        <a id='link'
           xmlns:x='http://example.com/ns'
           xmlns:y='http://example.com/ns'
           x:href='old'></a>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const ns = 'http://example.com/ns';
            const link = document.getElementById('link');
            const returned = link.setAttributeNS(ns, 'y:href', 'next');
            document.getElementById('result').textContent = [
              returned === undefined,
              link.getAttribute('x:href') === null,
              link.getAttribute('y:href'),
              link.getAttributeNS(ns, 'href')
            ].join(':');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "true:true:next:next")?;
    Ok(())
}

#[test]
fn element_set_attribute_ns_treats_empty_namespace_as_null() -> Result<()> {
    let html = r#"
        <div id='box' data-count='1'></div>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const box = document.getElementById('box');
            const returned = box.setAttributeNS('', 'data-count', '42');
            document.getElementById('result').textContent = [
              returned === undefined,
              box.getAttribute('data-count'),
              box.getAttributeNS(null, 'data-count'),
              box.hasAttributeNS('', 'DATA-COUNT')
            ].join(':');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "true:42:42:true")?;
    Ok(())
}

#[test]
fn element_set_attribute_ns_rejects_wrong_argument_count() -> Result<()> {
    let html = r#"
        <div id='box'></div>
        <button id='run'>run</button>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            document.getElementById('box').setAttributeNS('http://www.w3.org/1999/xlink', 'xlink:href');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    match h.click("#run") {
        Err(Error::ScriptRuntime(message)) => {
            assert!(
                message.contains("setAttributeNS requires exactly three arguments"),
                "unexpected runtime error message: {message}"
            );
        }
        other => panic!("expected runtime error, got: {other:?}"),
    }
    Ok(())
}

#[test]
fn element_set_attribute_ns_rejects_invalid_qualified_name() -> Result<()> {
    let html = r#"
        <div id='box'></div>
        <button id='run'>run</button>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            document.getElementById('box').setAttributeNS(
              'http://www.w3.org/1999/xlink',
              'xlink:href:extra',
              'v'
            );
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    match h.click("#run") {
        Err(Error::ScriptRuntime(message)) => {
            assert!(
                message.contains("InvalidCharacterError"),
                "unexpected runtime error message: {message}"
            );
        }
        other => panic!("expected runtime error, got: {other:?}"),
    }
    Ok(())
}

#[test]
fn element_set_attribute_ns_rejects_prefixed_name_without_namespace() -> Result<()> {
    let html = r#"
        <div id='box'></div>
        <button id='run'>run</button>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            document.getElementById('box').setAttributeNS(null, 'xlink:href', 'v');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    match h.click("#run") {
        Err(Error::ScriptRuntime(message)) => {
            assert!(
                message.contains("NamespaceError"),
                "unexpected runtime error message: {message}"
            );
        }
        other => panic!("expected runtime error, got: {other:?}"),
    }
    Ok(())
}

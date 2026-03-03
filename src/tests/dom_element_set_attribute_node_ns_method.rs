use super::*;

#[test]
fn element_set_attribute_node_ns_adds_and_replaces_namespaced_attr() -> Result<()> {
    let html = r#"
        <div id='one' xmlns:myns='http://www.mozilla.org/ns/specialspace' myns:special-align='utterleft'>one</div>
        <div id='two' xmlns:myns='http://www.mozilla.org/ns/specialspace'>two</div>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const ns = 'http://www.mozilla.org/ns/specialspace';
            const d1 = document.getElementById('one');
            const d2 = document.getElementById('two');
            const fromNode = d1.getAttributeNodeNS(ns, 'special-align');

            const replacedFirst = d2.setAttributeNodeNS(fromNode);
            const firstValue = d2.getAttributeNS(ns, 'special-align');

            const donorHost = document.createElement('div');
            donorHost.innerHTML =
              "<span xmlns:myns='http://www.mozilla.org/ns/specialspace' myns:special-align='center'></span>";
            const secondAttr = donorHost.firstElementChild.getAttributeNodeNS(
              ns,
              'special-align'
            );
            const replacedSecond = d2.setAttributeNodeNS(secondAttr);

            document.getElementById('result').textContent = [
              replacedFirst === null,
              firstValue,
              replacedSecond !== null ? replacedSecond.value : 'none',
              replacedSecond !== null ? replacedSecond.ownerElement === null : false,
              secondAttr.ownerElement === d2,
              d2.getAttributeNS(ns, 'special-align')
            ].join(':');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "true:utterleft:utterleft:true:true:center")?;
    Ok(())
}

#[test]
fn element_set_attribute_node_ns_replaces_by_namespace_and_local_name() -> Result<()> {
    let html = r#"
        <div id='box' xmlns:a='http://example.com/ns' xmlns:b='http://example.com/ns' a:flag='old'></div>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const ns = 'http://example.com/ns';
            const box = document.getElementById('box');
            const donorHost = document.createElement('div');
            donorHost.innerHTML =
              "<span xmlns:b='http://example.com/ns' b:flag='new'></span>";
            const attr = donorHost.firstElementChild.getAttributeNodeNS(ns, 'flag');

            const replaced = box.setAttributeNodeNS(attr);
            document.getElementById('result').textContent = [
              replaced !== null,
              replaced !== null ? replaced.name : 'none',
              replaced !== null ? replaced.value : 'none',
              box.getAttribute('a:flag') === null,
              box.getAttribute('b:flag'),
              box.getAttributeNS(ns, 'flag')
            ].join(':');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "true:a:flag:old:true:new:new")?;
    Ok(())
}

#[test]
fn element_set_attribute_node_ns_rejects_non_attr_argument() -> Result<()> {
    let html = r#"
        <div id='target'></div>
        <button id='run'>run</button>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            document.getElementById('target').setAttributeNodeNS({ name: 'xlink:href', value: 'v' });
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    match h.click("#run") {
        Err(Error::ScriptRuntime(message)) => {
            assert!(
                message.contains("setAttributeNodeNS argument must be an Attr"),
                "unexpected runtime error message: {message}"
            );
        }
        other => panic!("expected runtime error, got: {other:?}"),
    }
    Ok(())
}

#[test]
fn element_set_attribute_node_ns_rejects_wrong_argument_count() -> Result<()> {
    let html = r#"
        <div id='target'></div>
        <button id='run'>run</button>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const attr = document.createAttribute('data-x');
            document.getElementById('target').setAttributeNodeNS(attr, attr);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    match h.click("#run") {
        Err(Error::ScriptRuntime(message)) => {
            assert!(
                message.contains("setAttributeNodeNS requires exactly one argument"),
                "unexpected runtime error message: {message}"
            );
        }
        other => panic!("expected runtime error, got: {other:?}"),
    }
    Ok(())
}

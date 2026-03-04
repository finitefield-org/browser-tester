use super::*;

#[test]
fn named_node_map_get_named_item_item_and_live_length() -> Result<()> {
    let html = r#"
        <div id='box' class='green'></div>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const box = document.getElementById('box');
            const attrs = box.attributes;
            const beforeLen = box.attributes.length;
            const idAttr = attrs.getNamedItem('ID');
            const first = attrs.item(0);

            box.setAttribute('data-x', '1');
            const afterLen = box.attributes.length;
            const missing = attrs.getNamedItem('missing') === null;
            const dataValue = attrs.getNamedItem('data-x').value;

            document.getElementById('result').textContent = [
              idAttr !== null ? idAttr.value === 'box' : false,
              first !== null,
              beforeLen,
              afterLen,
              missing,
              dataValue
            ].join(':');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "true:true:2:3:true:1")?;
    Ok(())
}

#[test]
fn named_node_map_set_named_item_adds_and_replaces() -> Result<()> {
    let html = r#"
        <div id='box'></div>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const box = document.getElementById('box');
            const attrs = box.attributes;

            const first = document.createAttribute('data-state');
            first.value = 'ready';
            const replacedFirst = attrs.setNamedItem(first);

            const second = document.createAttribute('DATA-STATE');
            second.value = 'done';
            const replacedSecond = attrs.setNamedItem(second);

            document.getElementById('result').textContent = [
              replacedFirst === null,
              replacedSecond !== null ? replacedSecond.value : 'none',
              replacedSecond !== null ? replacedSecond.ownerElement === null : false,
              second.ownerElement === box,
              box.getAttribute('data-state')
            ].join(':');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "true:ready:true:true:done")?;
    Ok(())
}

#[test]
fn named_node_map_remove_named_item_returns_removed_attr_and_throws_when_missing() -> Result<()> {
    let html = r#"
        <div id='box' lang='en-US'></div>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const box = document.getElementById('box');
            const attrs = box.attributes;
            const removed = attrs.removeNamedItem('lang');

            let threw = false;
            try {
              attrs.removeNamedItem('lang');
            } catch (e) {
              threw = String(e).includes('NotFoundError');
            }

            document.getElementById('result').textContent = [
              removed.name,
              removed.value,
              removed.ownerElement === null,
              box.hasAttribute('lang'),
              threw
            ].join(':');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "lang:en-US:true:false:true")?;
    Ok(())
}

#[test]
fn named_node_map_namespaced_get_and_remove() -> Result<()> {
    let html = r#"
        <div id='box' xmlns:spec='http://example.com/ns' spec:align='left'></div>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const ns = 'http://example.com/ns';
            const box = document.getElementById('box');
            const attrs = box.attributes;
            const found = attrs.getNamedItemNS(ns, 'align');
            const removed = attrs.removeNamedItemNS(ns, 'align');

            let threw = false;
            try {
              attrs.removeNamedItemNS(ns, 'align');
            } catch (e) {
              threw = String(e).includes('NotFoundError');
            }

            document.getElementById('result').textContent = [
              found !== null ? found.name : 'none',
              removed !== null ? removed.value : 'none',
              removed !== null ? removed.ownerElement === null : false,
              box.getAttributeNS(ns, 'align') === null,
              threw
            ].join(':');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "spec:align:left:true:true:true")?;
    Ok(())
}

#[test]
fn named_node_map_set_named_item_ns_adds_and_replaces() -> Result<()> {
    let html = r#"
        <div id='one' xmlns:m='http://example.com/ns' m:flag='old'></div>
        <div id='two' xmlns:m='http://example.com/ns'></div>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const ns = 'http://example.com/ns';
            const one = document.getElementById('one');
            const two = document.getElementById('two');

            const donor = one.attributes.getNamedItemNS(ns, 'flag');
            const replacedFirst = two.attributes.setNamedItemNS(donor);
            const firstValue = two.getAttributeNS(ns, 'flag');

            const host = document.createElement('div');
            host.innerHTML =
              "<span xmlns:m='http://example.com/ns' m:flag='new'></span>";
            const secondAttr = host.firstElementChild.getAttributeNodeNS(ns, 'flag');
            const replacedSecond = two.attributes.setNamedItemNS(secondAttr);

            document.getElementById('result').textContent = [
              replacedFirst === null,
              firstValue,
              replacedSecond !== null ? replacedSecond.value : 'none',
              replacedSecond !== null ? replacedSecond.ownerElement === null : false,
              secondAttr.ownerElement === two,
              two.getAttributeNS(ns, 'flag')
            ].join(':');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "true:old:old:true:true:new")?;
    Ok(())
}

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

#[test]
fn named_node_map_raw_getter_methods_and_iterators_are_receiver_aware_work() -> Result<()> {
    let html = r#"
        <div id='box' class='green' data-state='ready'></div>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const attrs = document.getElementById('box').attributes;
            const item = attrs.item;
            const getNamedItem = attrs.getNamedItem;
            const values = attrs.values;
            const entries = attrs.entries;
            const iter = attrs[Symbol.iterator];

            let incompatible = false;
            try {
              Object.create(attrs).getNamedItem('class');
            } catch (e) {
              incompatible = String(e).includes('incompatible receiver');
            }

            document.getElementById('result').textContent = [
              item.call(attrs, 0).name,
              getNamedItem.call(attrs, 'class').value,
              Array.from(values.call(attrs)).map((attr) => attr.name).join(','),
              Array.from(entries.call(attrs))
                .map((pair) => pair[0] + ':' + pair[1].name)
                .join(','),
              Array.from(iter.call(attrs)).map((attr) => attr.value).join(','),
              typeof attrs.getNamedItem,
              typeof attrs.values,
              incompatible
            ].join('|');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "class|green|class,data-state,id|0:class,1:data-state,2:id|green,ready,box|function|function|true",
    )?;
    Ok(())
}

#[test]
fn named_node_map_named_property_collisions_follow_builtin_visibility_work() -> Result<()> {
    let html = r#"
        <div normal='ok' item='one' getNamedItem='two' values='three' toString='four' constructor='five' length='six'></div>
        <p id='result'></p>
        <script>
          const attrs = document.querySelector('div').attributes;
          const assigned = Object.assign({}, attrs);
          const spread = { ...attrs };

          document.getElementById('result').textContent = [
            typeof attrs.item,
            typeof attrs.getNamedItem,
            typeof attrs.values,
            typeof attrs['toString'],
            typeof attrs['constructor'],
            attrs.normal.value,
            Object.keys(attrs).includes('item'),
            Object.keys(attrs).includes('values'),
            Object.keys(attrs).includes('normal'),
            Object.getOwnPropertyNames(attrs).includes('toString'),
            Object.getOwnPropertyNames(attrs).includes('constructor'),
            Object.hasOwn(attrs, 'item'),
            Object.hasOwn(attrs, 'toString'),
            Object.hasOwn(attrs, 'constructor'),
            Object.getOwnPropertyDescriptor(attrs, 'item') === undefined,
            Object.getOwnPropertyDescriptor(attrs, 'values') === undefined,
            Object.getOwnPropertyDescriptor(attrs, 'toString') === undefined,
            attrs.length,
            Object.getOwnPropertyDescriptor(attrs, 'length').value,
            assigned.normal.value,
            'item' in assigned,
            'values' in spread
          ].join('|');
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text(
        "#result",
        "function|function|function|function|function|ok|false|false|true|false|false|false|false|false|true|true|true|7|7|ok|false|false",
    )?;
    Ok(())
}

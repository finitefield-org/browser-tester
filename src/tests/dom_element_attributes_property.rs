use super::*;

#[test]
fn element_attributes_returns_live_named_node_map() -> Result<()> {
    let html = r#"
        <p id='paragraph' class='green' contenteditable>Sample Paragraph</p>
        <button id='run'>run</button>
        <pre id='result'></pre>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const paragraph = document.getElementById('paragraph');
            const attrs = paragraph.attributes;
            const sameRef = attrs === paragraph.attributes;
            const firstLength = paragraph.attributes.length;

            paragraph.setAttribute('data-state', 'ready');
            const afterLength = paragraph.attributes.length;
            let seen = '';
            let ownerMatch = true;
            for (const attr of attrs) {
              seen += `${attr.name}:${attr.value}|`;
              ownerMatch = ownerMatch && attr.ownerElement === paragraph;
            }

            document.getElementById('result').textContent = [
              sameRef,
              typeof attrs.map,
              firstLength,
              afterLength,
              seen.includes('data-state:ready'),
              ownerMatch
            ].join(':');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "true:undefined:3:4:true:true")?;
    Ok(())
}

#[test]
fn element_attributes_is_iterable_and_yields_attr_nodes() -> Result<()> {
    let html = r#"
        <p id='paragraph' class='green' contenteditable>Sample Paragraph</p>
        <button id='run'>run</button>
        <pre id='result'></pre>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const paragraph = document.getElementById('paragraph');
            let output = '';
            for (const attr of paragraph.attributes) {
              output += `${attr.name}->${attr.value}|`;
            }
            document.getElementById('result').textContent = output.slice(0, -1);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "class->green|contenteditable->true|id->paragraph",
    )?;
    Ok(())
}

#[test]
fn element_attributes_own_keys_and_descriptors_track_live_entries_work() -> Result<()> {
    let html = r#"
        <p id='paragraph' class='green' contenteditable>Sample Paragraph</p>
        <button id='run'>run</button>
        <pre id='result'></pre>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const paragraph = document.getElementById('paragraph');
            const attrs = paragraph.attributes;

            const firstDesc = Object.getOwnPropertyDescriptor(attrs, '0');
            const classDesc = Object.getOwnPropertyDescriptor(attrs, 'class');
            const before = [
              String(Object.keys(attrs).includes('0')),
              String(Object.keys(attrs).includes('class')),
              String(Object.getOwnPropertyNames(attrs).includes('length')),
              String(Reflect.ownKeys(attrs).includes('class')),
              String(Object.hasOwn(attrs, 'class')),
              firstDesc.value.name,
              classDesc.value.value
            ].join(':');

            Object.defineProperty(attrs, 'class', {
              value: 'shadow',
              enumerable: true,
              configurable: true
            });

            const shadow = [
              String(attrs.class === 'shadow'),
              String(Object.getOwnPropertyDescriptor(attrs, 'class').value === 'shadow'),
              String(Object.keys(attrs).includes('class'))
            ].join(':');

            delete attrs.class;
            paragraph.setAttribute('data-state', 'ready');

            const dataDesc = Object.getOwnPropertyDescriptor(attrs, 'data-state');
            const after = [
              attrs.class.value,
              attrs['data-state'].value,
              String(Object.keys(attrs).includes('data-state')),
              String(Object.hasOwn(attrs, 'data-state')),
              dataDesc.value.value,
              Object.getOwnPropertyDescriptor(attrs, '0').value.name
            ].join(':');

            document.getElementById('result').textContent = [before, shadow, after].join('|');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "true:true:true:true:true:class:green|true:true:true|green:ready:true:true:ready:class",
    )?;
    Ok(())
}

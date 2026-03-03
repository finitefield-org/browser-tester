use super::*;

#[test]
fn element_prepend_prepending_element_matches_mdn_example() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const div = document.createElement('div');
            const p = document.createElement('p');
            const span = document.createElement('span');
            div.append(p);
            div.prepend(span);

            document.getElementById('result').textContent = [
              div.childNodes.length,
              div.childNodes[0].tagName,
              div.childNodes[1].tagName
            ].join(':');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "2:SPAN:P")?;
    Ok(())
}

#[test]
fn element_prepend_prepending_text_matches_mdn_example() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const div = document.createElement('div');
            div.append('Some text');
            div.prepend('Headline: ');

            document.getElementById('result').textContent = [
              div.textContent,
              div.childNodes.length,
              div.childNodes[0].nodeName,
              div.childNodes[1].nodeName
            ].join(':');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "Headline: Some text:2:#text:#text")?;
    Ok(())
}

#[test]
fn element_prepend_prepending_element_and_text_matches_mdn_example() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const div = document.createElement('div');
            const p = document.createElement('p');
            div.prepend('Some text', p);

            document.getElementById('result').textContent = [
              div.childNodes.length,
              div.childNodes[0].nodeName,
              div.childNodes[1].tagName,
              div.textContent
            ].join(':');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "2:#text:P:Some text")?;
    Ok(())
}

#[test]
fn element_prepend_returns_undefined_and_preserves_argument_order() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const div = document.createElement('div');
            const existing = document.createElement('p');
            existing.id = 'existing';
            div.append(existing);

            const a = document.createElement('a');
            const b = document.createElement('b');
            const returned = div.prepend(a, 'x', b);

            document.getElementById('result').textContent = [
              returned === undefined,
              div.childNodes.length,
              div.childNodes[0].tagName,
              div.childNodes[1].nodeName,
              div.childNodes[2].tagName,
              div.childNodes[3].id
            ].join(':');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "true:4:A:#text:B:existing")?;
    Ok(())
}

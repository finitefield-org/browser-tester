use super::*;

#[test]
fn element_append_appending_element_matches_mdn_example() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const div = document.createElement('div');
            const p = document.createElement('p');
            const returned = div.append(p);
            document.getElementById('result').textContent = [
              div.childNodes.length,
              div.childNodes[0].tagName,
              returned === undefined
            ].join(':');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "1:P:true")?;
    Ok(())
}

#[test]
fn element_append_appending_text_matches_mdn_example() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const div = document.createElement('div');
            div.append('Some text');
            document.getElementById('result').textContent = [
              div.textContent,
              div.childNodes.length,
              div.childNodes[0].nodeName
            ].join(':');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "Some text:1:#text")?;
    Ok(())
}

#[test]
fn element_append_appending_two_strings_adds_two_text_nodes() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const div = document.createElement('div');
            div.append('A', 'B');
            document.getElementById('result').textContent = [
              div.childNodes.length,
              div.childNodes[0].nodeName,
              div.childNodes[1].nodeName,
              div.textContent
            ].join(':');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "2:#text:#text:AB")?;
    Ok(())
}

#[test]
fn element_append_appending_element_and_text_matches_mdn_example() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const div = document.createElement('div');
            const p = document.createElement('p');
            div.append('Some text', p);
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
fn element_append_differs_from_append_child_return_and_accepts_multiple_items() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const div = document.createElement('div');
            const a = document.createElement('a');
            const b = document.createElement('b');
            const p = document.createElement('p');

            const appendReturned = div.append(a, 'x', b);
            const appendChildReturned = div.appendChild(p);
            document.getElementById('result').textContent = [
              appendReturned === undefined,
              appendChildReturned === p,
              div.childNodes.length,
              div.childNodes[1].nodeName,
              div.childNodes[3].tagName
            ].join(':');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "true:true:4:#text:P")?;
    Ok(())
}

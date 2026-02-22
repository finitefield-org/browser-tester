use super::*;

#[test]
fn p_has_implicit_paragraph_role_and_align_roundtrip_works() -> Result<()> {
    let html = r#"
        <p id='one'>Geckos are usually nocturnal lizards.</p>
        <button id='run' type='button'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const p = document.getElementById('one');
            const initial =
              p.role + ':' +
              p.tagName + ':' +
              p.textContent.trim();

            p.align = 'center';
            const alignByProp = p.align + ':' + p.getAttribute('align');

            p.setAttribute('align', 'right');
            const alignByAttr = p.align + ':' + p.getAttribute('align');

            p.role = 'note';
            const assignedRole = p.role + ':' + p.getAttribute('role');

            p.removeAttribute('role');
            const restoredRole = p.role + ':' + (p.getAttribute('role') === null);

            document.getElementById('result').textContent =
              initial + '|' + alignByProp + '|' + alignByAttr + '|' + assignedRole + '|' + restoredRole;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "paragraph:P:Geckos are usually nocturnal lizards.|center:center|right:right|note:note|paragraph:true",
    )?;
    Ok(())
}

#[test]
fn p_optional_end_tag_parsing_auto_closes_before_block_and_paragraph_elements() -> Result<()> {
    let html = r#"
        <div id='container'>
          <p id='first'>First paragraph
          <p id='second'>Second paragraph
          <div id='box'>Block sibling</div>
          <p id='third'>Third paragraph
          <h2 id='heading'>Heading sibling</h2>
        </div>
        <button id='run' type='button'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const children = document.querySelectorAll('#container > *');
            const paragraphs = document.querySelectorAll('#container > p');
            const nestedBlockCount = document.querySelectorAll('#first #box').length;

            document.getElementById('result').textContent =
              children.length + ':' +
              paragraphs.length + ':' +
              paragraphs[0].textContent.trim() + ':' +
              paragraphs[1].textContent.trim() + ':' +
              paragraphs[2].textContent.trim() + ':' +
              document.getElementById('box').tagName + ':' +
              document.getElementById('heading').tagName + ':' +
              nestedBlockCount;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "5:3:First paragraph:Second paragraph:Third paragraph:DIV:H2:0",
    )?;
    Ok(())
}

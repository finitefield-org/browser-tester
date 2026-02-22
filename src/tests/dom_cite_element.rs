use super::*;

#[test]
fn cite_marks_creative_work_title_in_figure_caption() -> Result<()> {
    let html = r#"
        <figure>
          <blockquote>
            <p>
              It was a bright cold day in April, and the clocks were striking thirteen.
            </p>
          </blockquote>
          <figcaption>
            First sentence in
            <cite id='title'>
              <a id='book-link' href='http://www.george-orwell.org/1984/0.html'>Nineteen Eighty-Four</a>
            </cite>
            by George Orwell (Part 1, Chapter 1).
          </figcaption>
        </figure>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const title = document.getElementById('title');
            const link = document.getElementById('book-link');
            document.getElementById('result').textContent =
              title.tagName + ':' +
              title.role + ':' +
              title.textContent.trim() + ':' +
              link.href + ':' +
              document.querySelectorAll('figcaption cite a').length;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "CITE::Nineteen Eighty-Four:http://www.george-orwell.org/1984/0.html:1",
    )?;
    Ok(())
}

#[test]
fn cite_role_attribute_overrides_and_remove_restores_empty_implicit_role() -> Result<()> {
    let html = r#"
        <p>More information can be found in <cite id='ref'>[ISO-0000]</cite>.</p>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const ref = document.getElementById('ref');
            const initial = ref.role;
            ref.role = 'note';
            const assigned = ref.role + ':' + ref.getAttribute('role');
            ref.removeAttribute('role');
            const restored = ref.role + ':' + (ref.getAttribute('role') === null);
            document.getElementById('result').textContent =
              initial + '|' + assigned + '|' + restored + '|' + ref.textContent;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "|note:note|:true|[ISO-0000]")?;
    Ok(())
}

use super::*;

#[test]
fn hgroup_has_implicit_group_role_and_keeps_heading_plus_paragraphs() -> Result<()> {
    let html = r#"
        <hgroup id='book-title'>
          <h1>Frankenstein</h1>
          <p>Or: The Modern Prometheus</p>
        </hgroup>
        <button id='run' type='button'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const group = document.getElementById('book-title');
            const heading = group.querySelector('h1');
            const subtitles = group.querySelectorAll('p');
            document.getElementById('result').textContent =
              group.role + ':' +
              group.tagName + ':' +
              heading.role + ':' +
              heading.textContent.trim() + ':' +
              subtitles.length + ':' +
              subtitles[0].textContent.trim();
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "group:HGROUP:heading:Frankenstein:1:Or: The Modern Prometheus",
    )?;
    Ok(())
}

#[test]
fn hgroup_role_assignment_roundtrip_and_document_queries_work() -> Result<()> {
    let html = r#"
        <hgroup id='title-block'>
          <p>Last Updated 12 July 2022</p>
          <h1>HTML: Living Standard</h1>
          <p>Snapshot</p>
        </hgroup>
        <button id='run' type='button'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const group = document.getElementById('title-block');
            const before = group.role + ':' + document.querySelectorAll('hgroup').length;

            group.role = 'none';
            const assigned = group.role + ':' + group.getAttribute('role');
            group.removeAttribute('role');
            const restored = group.role + ':' + (group.getAttribute('role') === null);

            document.getElementById('result').textContent =
              before + '|' + assigned + '|' + restored;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "group:1|none:none|group:true")?;
    Ok(())
}

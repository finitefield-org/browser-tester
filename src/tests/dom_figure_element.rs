use super::*;

#[test]
fn figure_with_caption_has_implicit_figure_role() -> Result<()> {
    let html = r#"
        <figure id='hero'>
          <img
            src="/shared-assets/images/examples/elephant.jpg"
            alt="Elephant at sunset">
          <figcaption>An elephant at sunset</figcaption>
        </figure>
        <button id='run' type='button'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const hero = document.getElementById('hero');
            document.getElementById('result').textContent =
              hero.role + ':' +
              hero.tagName + ':' +
              hero.querySelectorAll('figcaption').length + ':' +
              hero.querySelector('figcaption').textContent.trim() + ':' +
              hero.querySelector('img').getAttribute('alt');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "figure:FIGURE:1:An elephant at sunset:Elephant at sunset",
    )?;
    Ok(())
}

#[test]
fn figure_role_roundtrip_and_first_figcaption_resolution_work() -> Result<()> {
    let html = r#"
        <figure id='snippet'>
          <figcaption id='cap1'>First caption</figcaption>
          <pre>const x = 1;</pre>
          <figcaption id='cap2'>Second caption</figcaption>
        </figure>
        <button id='run' type='button'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const snippet = document.getElementById('snippet');
            const placement =
              (document.querySelector('#snippet > figcaption:first-child') !== null) + ':' +
              (document.querySelector('#snippet > figcaption:last-child') !== null);

            const initial = snippet.role;
            snippet.role = 'region';
            const assigned = snippet.role + ':' + snippet.getAttribute('role');
            snippet.removeAttribute('role');
            const restored = snippet.role + ':' + (snippet.getAttribute('role') === null);

            document.getElementById('result').textContent =
              placement + '|' +
              document.querySelectorAll('#snippet > figcaption').length + ':' +
              document.getElementById('cap1').textContent.trim() + '|' +
              initial + '|' + assigned + '|' + restored;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "true:true|2:First caption|figure|region:region|figure:true",
    )?;
    Ok(())
}

use super::*;

#[test]
fn figcaption_describes_figure_and_has_no_implicit_role() -> Result<()> {
    let html = r#"
        <figure>
          <img
            src="/shared-assets/images/examples/elephant.jpg"
            alt="Elephant at sunset">
          <figcaption id='caption'>An elephant at sunset</figcaption>
        </figure>
        <button id='run' type='button'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const caption = document.getElementById('caption');
            document.getElementById('result').textContent =
              caption.role + ':' +
              caption.tagName + ':' +
              caption.textContent.trim() + ':' +
              document.querySelectorAll('figure > figcaption').length + ':' +
              document.querySelector('figure > img').getAttribute('alt');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        ":FIGCAPTION:An elephant at sunset:1:Elephant at sunset",
    )?;
    Ok(())
}

#[test]
fn figcaption_can_be_first_or_last_child_and_role_roundtrip_works() -> Result<()> {
    let html = r#"
        <figure id='first'>
          <figcaption id='first-caption'>Caption first</figcaption>
          <img src='/a.jpg' alt='A'>
        </figure>
        <figure id='last'>
          <img src='/b.jpg' alt='B'>
          <figcaption id='last-caption'>Caption last</figcaption>
        </figure>
        <button id='run' type='button'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const firstPlacement = document.querySelector('#first > figcaption:first-child') !== null;
            const lastPlacement = document.querySelector('#last > figcaption:last-child') !== null;

            const caption = document.getElementById('last-caption');
            const initial = caption.role;
            caption.role = 'presentation';
            const assigned = caption.role + ':' + caption.getAttribute('role');
            caption.removeAttribute('role');
            const restored = caption.role + ':' + (caption.getAttribute('role') === null);

            document.getElementById('result').textContent =
              firstPlacement + ':' + lastPlacement + '|' +
              initial + '|' + assigned + '|' + restored + '|' +
              document.getElementById('first-caption').textContent.trim() + ':' +
              caption.textContent.trim();
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "true:true||presentation:presentation|:true|Caption first:Caption last",
    )?;
    Ok(())
}

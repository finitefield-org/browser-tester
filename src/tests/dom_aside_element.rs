use super::*;

#[test]
fn aside_implicit_complementary_role_and_nested_usage_work() -> Result<()> {
    let html = r#"
        <article id='post'>
          <p>The Disney movie <cite>The Little Mermaid</cite> was first released to theatres in 1989.</p>
          <aside id='fact'>
            <p>The movie earned $87 million during its initial release.</p>
          </aside>
          <p>More info about the movie...</p>
        </article>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const aside = document.getElementById('fact');
            const asides = document.querySelectorAll('article aside');
            document.getElementById('result').textContent =
              aside.role + ':' +
              asides.length + ':' +
              aside.querySelector('p').textContent;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "complementary:1:The movie earned $87 million during its initial release.",
    )?;
    Ok(())
}

#[test]
fn aside_role_attribute_overrides_and_remove_restores_implicit_role() -> Result<()> {
    let html = r#"
        <aside id='sidebox'>
          <p>Callout</p>
        </aside>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const sidebox = document.getElementById('sidebox');
            const initial = sidebox.role;
            sidebox.role = 'note';
            const assigned = sidebox.role + ':' + sidebox.getAttribute('role');
            sidebox.removeAttribute('role');
            const restored = sidebox.role + ':' + (sidebox.getAttribute('role') === null);
            document.getElementById('result').textContent =
              initial + '|' + assigned + '|' + restored;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "complementary|note:note|complementary:true")?;
    Ok(())
}

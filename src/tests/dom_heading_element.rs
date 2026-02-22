use super::*;

#[test]
fn h1_to_h6_have_implicit_heading_role() -> Result<()> {
    let html = r#"
        <h1 id='h1'>Beetles</h1>
        <h2 id='h2'>External morphology</h2>
        <h3 id='h3'>Head</h3>
        <h4 id='h4'><em>Mouthparts</em></h4>
        <h5 id='h5'>Layer 5</h5>
        <h6 id='h6'>Layer 6</h6>
        <button id='run' type='button'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const h1 = document.getElementById('h1');
            const h2 = document.getElementById('h2');
            const h3 = document.getElementById('h3');
            const h4 = document.getElementById('h4');
            const h5 = document.getElementById('h5');
            const h6 = document.getElementById('h6');
            const total = document.querySelectorAll('h1, h2, h3, h4, h5, h6').length;
            document.getElementById('result').textContent =
              total + ':' +
              h1.role + ':' +
              h2.role + ':' +
              h3.role + ':' +
              h4.role + ':' +
              h5.role + ':' +
              h6.role + ':' +
              h4.textContent.trim();
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "6:heading:heading:heading:heading:heading:heading:Mouthparts",
    )?;
    Ok(())
}

#[test]
fn heading_role_roundtrip_and_global_attrs_work() -> Result<()> {
    let html = r#"
        <h2 id='target' lang='en' dir='ltr'>Heading level 2</h2>
        <button id='run' type='button'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const target = document.getElementById('target');
            const initial =
              target.role + ':' +
              target.getAttribute('lang') + ':' +
              target.dir + ':' +
              target.tagName;

            target.role = 'tab';
            const assigned = target.role + ':' + target.getAttribute('role');
            target.removeAttribute('role');
            const restored = target.role + ':' + (target.getAttribute('role') === null);

            target.setAttribute('lang', 'ja');
            target.dir = 'rtl';
            const globals =
              target.getAttribute('lang') + ':' +
              target.getAttribute('lang') + ':' +
              target.dir + ':' +
              target.getAttribute('dir');

            document.getElementById('result').textContent =
              initial + '|' + assigned + '|' + restored + '|' + globals;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "heading:en:ltr:H2|tab:tab|heading:true|ja:ja:rtl:rtl",
    )?;
    Ok(())
}

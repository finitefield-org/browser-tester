use super::*;

#[test]
fn rt_has_no_implicit_role_and_supports_role_override_in_ruby() -> Result<()> {
    let html = r#"
        <ruby id='word'>
          漢<rp>(</rp><rt id='kan'>kan</rt><rp>)</rp>
          字<rp>(</rp><rt id='ji'>ji</rt><rp>)</rp>
        </ruby>
        <button id='run' type='button'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const kan = document.getElementById('kan');
            const ji = document.getElementById('ji');
            const initial =
              kan.role + ':' +
              kan.tagName + ':' +
              kan.textContent.trim() + ':' +
              ji.textContent.trim() + ':' +
              document.querySelectorAll('#word > rt').length + ':' +
              document.querySelectorAll('#word > rp').length;

            kan.role = 'note';
            const assigned = kan.role + ':' + kan.getAttribute('role');
            kan.removeAttribute('role');
            const restored = kan.role + ':' + (kan.getAttribute('role') === null);

            document.getElementById('result').textContent =
              initial + '|' + assigned + '|' + restored;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", ":RT:kan:ji:2:4|note:note|:true")?;
    Ok(())
}

#[test]
fn rt_optional_end_tag_closes_before_rt_and_rp() -> Result<()> {
    let html = r#"
        <ruby id='r'>
          字<rt id='a'>ji
          <rt id='b'>go
          <rp id='fallback'>(</rp>
          <rt id='c'>ku
        </ruby>
        <button id='run' type='button'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const a = document.getElementById('a');
            const b = document.getElementById('b');
            const c = document.getElementById('c');

            document.getElementById('result').textContent =
              document.querySelectorAll('#r > rt').length + ':' +
              document.querySelectorAll('#r > rp').length + ':' +
              document.querySelectorAll('#a > rt').length + ':' +
              document.querySelectorAll('#b > rp').length + ':' +
              a.textContent.trim() + ':' +
              b.textContent.trim() + ':' +
              c.textContent.trim() + ':' +
              c.tagName;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "3:1:0:0:ji:go:ku:RT")?;
    Ok(())
}

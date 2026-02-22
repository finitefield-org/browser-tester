use super::*;

#[test]
fn rp_optional_end_tag_closes_before_rt_and_preserves_fallback_text() -> Result<()> {
    let html = r#"
        <ruby id='ruby'>
          漢<rp id='open'>(<rt id='kan'>kan</rt><rp id='close'>)<rt id='ji'>ji</rt>
        </ruby>
        <button id='run' type='button'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const ruby = document.getElementById('ruby');
            const open = document.getElementById('open');
            const close = document.getElementById('close');

            const initial =
              document.querySelectorAll('#ruby > rp').length + ':' +
              document.querySelectorAll('#ruby > rt').length + ':' +
              document.querySelectorAll('#open > rt').length + ':' +
              open.textContent.trim() + ':' +
              close.textContent.trim() + ':' +
              open.role + ':' +
              ruby.tagName;

            open.role = 'note';
            const assignedRole = open.role + ':' + open.getAttribute('role');
            open.removeAttribute('role');
            const restoredRole = open.role + ':' + (open.getAttribute('role') === null);

            document.getElementById('result').textContent =
              initial + '|' + assignedRole + '|' + restoredRole;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "2:2:0:(:)::RUBY|note:note|:true")?;
    Ok(())
}

#[test]
fn rp_optional_end_tag_closes_before_another_rp() -> Result<()> {
    let html = r#"
        <ruby id='ruby2'>
          字<rp id='a'>(<rp id='b'>[</rp><rt id='rt'>ji</rt><rp id='c'>)</rp>
        </ruby>
        <button id='run' type='button'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const children = document.querySelectorAll('#ruby2 > *');

            document.getElementById('result').textContent =
              document.querySelectorAll('#ruby2 > rp').length + ':' +
              document.querySelectorAll('#ruby2 > rt').length + ':' +
              document.querySelectorAll('#a > rp').length + ':' +
              children[0].tagName + ':' +
              children[1].tagName + ':' +
              children[2].tagName + ':' +
              children[3].tagName + ':' +
              document.getElementById('a').textContent.trim() + ':' +
              document.getElementById('b').textContent.trim() + ':' +
              document.getElementById('c').textContent.trim();
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "3:1:0:RP:RP:RT:RP:(:[:)")?;
    Ok(())
}

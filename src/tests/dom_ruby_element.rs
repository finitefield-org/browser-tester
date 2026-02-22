use super::*;

#[test]
fn ruby_character_and_word_annotations_preserve_structure_and_text() -> Result<()> {
    let html = r#"
        <ruby id='char'>
          漢 <rp>(</rp><rt>Kan</rt><rp>)</rp>
          字 <rp>(</rp><rt>ji</rt><rp>)</rp>
        </ruby>
        <ruby id='word'>明日 <rp>(</rp><rt>Ashita</rt><rp>)</rp></ruby>

        <button id='run' type='button'>run</button>
        <p id='result'></p>
        <script>
          const normalize = (node) => node.textContent.replace(/\s+/g, '').trim();
          document.getElementById('run').addEventListener('click', () => {
            const charRuby = document.getElementById('char');
            const wordRuby = document.getElementById('word');

            const initial =
              charRuby.tagName + ':' +
              charRuby.role + ':' +
              charRuby.querySelectorAll('rt').length + ':' +
              charRuby.querySelectorAll('rp').length + ':' +
              normalize(charRuby) + ':' +
              wordRuby.querySelectorAll('rt').length + ':' +
              normalize(wordRuby);

            charRuby.setAttribute('lang', 'ja');
            charRuby.title = 'reading';
            const updated =
              charRuby.title + ':' +
              charRuby.getAttribute('lang') + ':' +
              charRuby.getAttribute('title');

            document.getElementById('result').textContent = initial + '|' + updated;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "RUBY::2:4:漢(Kan)字(ji):1:明日(Ashita)|reading:ja:reading",
    )?;
    Ok(())
}

#[test]
fn ruby_role_override_roundtrips_and_nested_ruby_stays_queryable() -> Result<()> {
    let html = r#"
        <ruby id='outer'>
          東<rt>to</rt>京<rt>kyo</rt>
          <ruby id='inner'>都<rt>to</rt></ruby>
        </ruby>
        <button id='run' type='button'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const outer = document.getElementById('outer');
            const inner = document.getElementById('inner');
            const initial =
              outer.role + ':' +
              inner.role + ':' +
              document.querySelectorAll('#outer ruby').length + ':' +
              outer.querySelectorAll('rt').length + ':' +
              inner.querySelectorAll('rt').length;

            outer.role = 'group';
            const assigned = outer.role + ':' + outer.getAttribute('role');
            outer.removeAttribute('role');
            const restored = outer.role + ':' + (outer.getAttribute('role') === null);

            document.getElementById('result').textContent =
              initial + '|' + assigned + '|' + restored;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "::1:3:1|group:group|:true")?;
    Ok(())
}

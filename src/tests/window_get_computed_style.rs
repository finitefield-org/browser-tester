use super::*;

#[test]
fn get_computed_style_returns_live_values_and_property_aliases() -> Result<()> {
    let html = r#"
      <div id='box' style='color: blue; background-color: lime;'></div>
      <button id='run'>run</button>
      <p id='result'></p>
      <script>
        document.getElementById('run').addEventListener('click', () => {
          const box = document.getElementById('box');
          const styles = window.getComputedStyle(box);
          const before = styles.getPropertyValue('color');
          box.style.color = 'red';
          const after = styles.getPropertyValue('color');
          const alias = styles.color;
          const camel = styles.backgroundColor;
          const viaNullPseudo = window.getComputedStyle(box, null).getPropertyValue('color');
          document.getElementById('result').textContent =
            [before, after, alias, camel, viaNullPseudo].join(':');
        });
      </script>
    "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "blue:red:red:lime:red")?;
    Ok(())
}

#[test]
fn get_computed_style_supports_pseudo_argument_and_global_alias_chain() -> Result<()> {
    let html = r#"
      <style>
        #box::after { content: " rocks!"; color: green; }
      </style>
      <div id='box' style='color: blue;'></div>
      <button id='run'>run</button>
      <p id='result'></p>
      <script>
        document.getElementById('run').addEventListener('click', () => {
          const box = document.getElementById('box');
          const pseudo = getComputedStyle(box, '::after');
          const contentByMethod = pseudo.getPropertyValue('content');
          const contentByProp = pseudo.content;
          const pseudoColor = pseudo.getPropertyValue('color');
          const elementColor = getComputedStyle(box).getPropertyValue('color');
          document.getElementById('result').textContent =
            [contentByMethod, contentByProp, pseudoColor, elementColor].join(':');
        });
      </script>
    "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "\" rocks!\":\" rocks!\":green:blue")?;
    Ok(())
}

#[test]
fn get_computed_style_rejects_invalid_target_and_pseudo_values() -> Result<()> {
    let html = r#"
      <div id='box' style='color: blue;'></div>
      <button id='run'>run</button>
      <p id='result'></p>
      <script>
        document.getElementById('run').addEventListener('click', () => {
          const box = document.getElementById('box');
          const checks = [];

          try {
            window.getComputedStyle(1);
            checks.push('bad-target');
          } catch (e) {
            checks.push(String(e).includes('TypeError'));
          }

          try {
            window.getComputedStyle(box, 'before');
            checks.push('bad-pseudo');
          } catch (e) {
            checks.push(String(e).includes('pseudoElt'));
          }

          try {
            window.getComputedStyle(box, '::part(tab)');
            checks.push('bad-part');
          } catch (e) {
            checks.push(String(e).includes('pseudoElt'));
          }

          try {
            window.getComputedStyle(box, 42);
            checks.push('bad-type');
          } catch (e) {
            checks.push(String(e).includes('pseudoElt'));
          }

          document.getElementById('result').textContent = checks.join(':');
        });
      </script>
    "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "true:true:true:true")?;
    Ok(())
}

#[test]
fn computed_style_object_is_read_only() -> Result<()> {
    let html = r#"
      <div id='box' style='color: blue;'></div>
      <button id='run'>run</button>
      <p id='result'></p>
      <script>
        document.getElementById('run').addEventListener('click', () => {
          const box = document.getElementById('box');
          const styles = getComputedStyle(box);
          let readOnly = false;
          try {
            styles.color = 'black';
          } catch (e) {
            readOnly = String(e).includes('read-only');
          }
          document.getElementById('result').textContent =
            String(readOnly) + ':' + styles.getPropertyValue('color');
        });
      </script>
    "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "true:blue")?;
    Ok(())
}

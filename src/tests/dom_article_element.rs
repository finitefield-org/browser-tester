use super::*;

#[test]
fn article_implicit_role_and_nested_structure_work() -> Result<()> {
    let html = r#"
        <article id='forecast' class='forecast'>
          <h1>Weather forecast for Seattle</h1>
          <article id='day1' class='day-forecast'>
            <h2>03 March 2018</h2>
            <p>Rain.</p>
          </article>
          <article id='day2' class='day-forecast'>
            <h2>04 March 2018</h2>
            <p>Periods of rain.</p>
          </article>
        </article>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const articles = document.querySelectorAll('article');
            const forecast = document.getElementById('forecast');
            const day1 = document.getElementById('day1');
            const day2 = document.getElementById('day2');
            document.getElementById('result').textContent =
              articles.length + ':' +
              forecast.role + ':' +
              day1.role + ':' +
              day2.querySelector('h2').textContent + ':' +
              forecast.querySelectorAll('.day-forecast').length;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "3:article:article:04 March 2018:2")?;
    Ok(())
}

#[test]
fn article_role_attribute_overrides_and_remove_restores_implicit_role() -> Result<()> {
    let html = r#"
        <article id='entry'>
          <h2>Jurassic Park</h2>
          <section>
            <h3>Review</h3>
            <p>Dinos were great!</p>
          </section>
        </article>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const entry = document.getElementById('entry');
            const initial = entry.role;
            entry.role = 'region';
            const assigned = entry.role + ':' + entry.getAttribute('role');
            entry.removeAttribute('role');
            const restored = entry.role + ':' + (entry.getAttribute('role') === null);
            document.getElementById('result').textContent =
              initial + '|' + assigned + '|' + restored;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "article|region:region|article:true")?;
    Ok(())
}

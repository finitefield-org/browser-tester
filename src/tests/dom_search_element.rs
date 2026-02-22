use super::*;

#[test]
fn search_has_implicit_search_role_and_contains_search_controls() -> Result<()> {
    let html = r#"
        <header>
          <h1>Movie website</h1>
          <search id='site-search'>
            <form action='./search/'>
              <label for='movie'>Find a Movie</label>
              <input type='search' id='movie' name='q' value='gecko' />
              <button type='submit'>Search</button>
            </form>
          </search>
        </header>

        <main>
          <h2>Cars available for rent</h2>
          <search id='car-search' title='Cars'>
            <h3>Filter results</h3>
            <label for='query'>Find and filter your query</label>
            <input type='search' id='query' value='compact' />
            <label>
              <input type='checkbox' id='exact-only' checked />
              Exact matches only
            </label>
            <section>
              <h4>Results:</h4>
              <ul id='results'>
                <li>City Hatchback</li>
              </ul>
              <output id='no-results'></output>
            </section>
          </search>
        </main>

        <button id='run' type='button'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const siteSearch = document.getElementById('site-search');
            const carSearch = document.getElementById('car-search');

            document.getElementById('result').textContent =
              siteSearch.role + ':' +
              carSearch.role + ':' +
              siteSearch.tagName + ':' +
              siteSearch.querySelectorAll('input').length + ':' +
              carSearch.querySelectorAll('input').length + ':' +
              carSearch.querySelector('output').tagName + ':' +
              document.querySelectorAll('search').length + ':' +
              carSearch.getAttribute('title');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "search:search:SEARCH:1:2:OUTPUT:2:Cars")?;
    Ok(())
}

#[test]
fn search_role_override_and_global_attributes_roundtrip_work() -> Result<()> {
    let html = r#"
        <search id='filters' aria-label='Cars filters'>
          <h3>Filter results</h3>
          <label for='brand'>Brand</label>
          <input id='brand' type='search' value='Mazda' />
        </search>

        <button id='run' type='button'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const filters = document.getElementById('filters');
            const initial =
              filters.role + ':' +
              filters.getAttribute('aria-label') + ':' +
              filters.querySelector('h3').textContent.trim();

            filters.role = 'region';
            const assigned = filters.role + ':' + filters.getAttribute('role');

            filters.removeAttribute('role');
            const restored = filters.role + ':' + (filters.getAttribute('role') === null);

            filters.setAttribute('title', 'Cars');
            const titleSet = filters.getAttribute('title');
            filters.removeAttribute('title');
            const titleRemoved = filters.getAttribute('title') === null;

            document.getElementById('result').textContent =
              initial + '|' + assigned + '|' + restored + '|' +
              titleSet + ':' + titleRemoved;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "search:Cars filters:Filter results|region:region|search:true|Cars:true",
    )?;
    Ok(())
}

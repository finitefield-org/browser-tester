use super::*;

#[test]
fn address_implicit_group_role_and_contact_links_work() -> Result<()> {
    let html = r#"
        <p>Contact the author of this page:</p>
        <address id='contact'>
          <a id='mail' href='mailto:jim@example.com'>jim@example.com</a><br />
          <a id='tel' href='tel:+14155550132'>+1 (415) 555-0132</a>
        </address>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const contact = document.getElementById('contact');
            document.getElementById('result').textContent =
              contact.role + ':' +
              contact.tagName + ':' +
              document.querySelectorAll('address a').length;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "group:ADDRESS:2")?;

    h.click("#mail")?;
    h.click("#tel")?;
    assert_eq!(
        h.take_location_navigations(),
        vec![
            LocationNavigation {
                kind: LocationNavigationKind::Assign,
                from: "about:blank".to_string(),
                to: "mailto:jim@example.com".to_string(),
            },
            LocationNavigation {
                kind: LocationNavigationKind::Assign,
                from: "mailto:jim@example.com".to_string(),
                to: "tel:+14155550132".to_string(),
            },
        ]
    );
    Ok(())
}

#[test]
fn address_role_attribute_overrides_and_remove_restores_implicit_role() -> Result<()> {
    let html = r#"
        <article>
          <footer>
            <address id='author'>Contact author</address>
          </footer>
        </article>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const author = document.getElementById('author');
            const initial = author.role;
            author.role = 'contentinfo';
            const assigned = author.role + ':' + author.getAttribute('role');
            author.removeAttribute('role');
            const restored = author.role + ':' + (author.getAttribute('role') === null);
            document.getElementById('result').textContent =
              initial + '|' + assigned + '|' + restored;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "group|contentinfo:contentinfo|group:true")?;
    Ok(())
}

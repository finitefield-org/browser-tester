use super::*;

#[test]
fn form_implicit_role_and_role_assignment_roundtrip() -> Result<()> {
    let html = r#"
        <form id='target' name='signup'>
          <label for='email'>Email</label>
          <input id='email' name='email' type='email' required>
        </form>
        <button id='run' type='button'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const form = document.getElementById('target');
            const initial = form.role + ':' + form.tagName + ':' + form.getAttribute('name');
            form.role = 'search';
            const assigned = form.role + ':' + form.getAttribute('role');
            form.removeAttribute('role');
            const restored = form.role + ':' + (form.getAttribute('role') === null);
            document.getElementById('result').textContent =
              initial + '|' + assigned + '|' + restored;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "form:FORM:signup|search:search|form:true")?;
    Ok(())
}

#[test]
fn form_submission_attributes_and_request_submit_work() -> Result<()> {
    let html = r#"
        <form id='target' action='/subscribe' method='get' target='_blank' autocomplete='on' accept-charset='UTF-8' rel='search'>
          <input id='email' name='email' type='email' required value='seed@example.com'>
          <button id='submitter' type='submit'>Subscribe</button>
        </form>
        <button id='run' type='button'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const form = document.getElementById('target');
            const submitter = document.getElementById('submitter');
            let submits = 0;
            form.addEventListener('submit', (event) => {
              submits++;
              event.preventDefault();
            });

            const before =
              form.getAttribute('action') + ':' +
              form.getAttribute('method') + ':' +
              form.getAttribute('target') + ':' +
              form.getAttribute('autocomplete') + ':' +
              form.getAttribute('accept-charset') + ':' +
              form.getAttribute('rel') + ':' +
              form.hasAttribute('novalidate');

            form.setAttribute('method', 'post');
            form.setAttribute('enctype', 'multipart/form-data');
            form.setAttribute('target', '_self');
            form.setAttribute('novalidate', '');
            form.setAttribute('name', 'newsletter');

            const afterAttrs =
              form.getAttribute('method') + ':' +
              form.getAttribute('enctype') + ':' +
              form.getAttribute('target') + ':' +
              form.hasAttribute('novalidate') + ':' +
              form.getAttribute('name');

            form.requestSubmit(submitter);

            const formData = new FormData(form);
            const afterSubmit =
              submits + ':' +
              formData.get('email') + ':' +
              form.elements.length;

            document.getElementById('result').textContent =
              before + '|' + afterAttrs + '|' + afterSubmit;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "/subscribe:get:_blank:on:UTF-8:search:false|post:multipart/form-data:_self:true:newsletter|1:seed@example.com:2",
    )?;
    Ok(())
}

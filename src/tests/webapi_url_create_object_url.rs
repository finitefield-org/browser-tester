use super::*;

#[test]
fn url_create_object_url_returns_unique_blob_urls_and_revoke_returns_undefined() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const blob = new Blob(['hello'], { type: 'text/plain' });
            const url1 = URL.createObjectURL(blob);
            const url2 = URL.createObjectURL(blob);
            const revoked = URL.revokeObjectURL(url1);
            document.getElementById('result').textContent =
              url1.startsWith('blob:bt-') + ':' +
              url2.startsWith('blob:bt-') + ':' +
              (url1 !== url2) + ':' +
              (revoked === undefined);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "true:true:true:true")?;
    Ok(())
}

#[test]
fn url_create_object_url_requires_blob_argument() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            URL.createObjectURL('not-a-blob');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    let err = h
        .click("#run")
        .expect_err("URL.createObjectURL should reject non-Blob argument");
    match err {
        Error::ScriptRuntime(message) => {
            assert!(
                message.contains("URL.createObjectURL requires a Blob argument"),
                "unexpected runtime error message: {message}",
            );
        }
        other => panic!("unexpected error type: {other:?}"),
    }
    Ok(())
}

#[test]
fn url_create_object_url_parser_arity_error_is_stable() {
    let err = Harness::from_html("<script>URL.createObjectURL();</script>")
        .expect_err("URL.createObjectURL without arguments should fail to parse");
    match err {
        Error::ScriptParse(message) => {
            assert!(message.contains("URL.createObjectURL requires exactly one argument"));
        }
        other => panic!("unexpected error type: {other:?}"),
    }
}

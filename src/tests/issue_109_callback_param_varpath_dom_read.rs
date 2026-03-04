use super::*;

#[test]
fn issue_109_callback_param_terms_path_is_treated_as_plain_object_property() -> Result<()> {
    let html = r#"
      <p id='out'></p>
      <script>
        function buildFieldMap(fields) {
          const map = {};
          fields.forEach((field) => {
            map[field.key] = field;
          });
          return map;
        }

        function isSingleValue(field) {
          return !!field && field.terms.length === 1 && field.terms[0].type === 'value';
        }

        function singleValue(field) {
          if (!isSingleValue(field)) return null;
          return field.terms[0].value;
        }

        const fields = [
          { key: 'min', terms: [{ type: 'value', value: 9 }] }
        ];
        const map = buildFieldMap(fields);
        document.getElementById('out').textContent = String(singleValue(map.min));
      </script>
    "#;

    let harness = Harness::from_html(html)?;
    harness.assert_text("#out", "9")?;
    Ok(())
}

#[test]
fn issue_109_indexed_var_path_type_and_value_do_not_require_element_resolution() -> Result<()> {
    let html = r#"
      <p id='out'></p>
      <script>
        const state = {
          source: [{ type: 'hex', value: 'deadbeef' }]
        };
        const first = state.source[0];
        document.getElementById('out').textContent = first.type + ':' + first.value;
      </script>
    "#;

    let harness = Harness::from_html(html)?;
    harness.assert_text("#out", "hex:deadbeef")?;
    Ok(())
}

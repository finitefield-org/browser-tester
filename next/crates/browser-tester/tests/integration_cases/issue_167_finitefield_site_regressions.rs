use browser_tester::Harness;

#[test]
fn issue_167_reassigned_intl_number_format_is_used_by_page_code() -> browser_tester::Result<()> {
    let html = r#"
      <pre id="out"></pre>
      <script>
        Intl = {
          NumberFormat: function () {
            throw new Error("forced Intl failure");
          }
        };
        window.Intl = Intl;
        Intl.NumberFormat = function () {
          throw new Error("forced Intl failure");
        };

        function formatIndex(value, lang, minimumIntegerDigits) {
          const safeValue = Math.max(0, Number(value) || 0);
          try {
            return new Intl.NumberFormat(lang, {
              useGrouping: false,
              minimumIntegerDigits,
              maximumFractionDigits: 0
            }).format(safeValue);
          } catch (error) {
            const digits = String(Math.trunc(safeValue));
            return digits.padStart(minimumIntegerDigits, "0");
          }
        }

        const lines = ["A", "B"].map((label, index) => {
          return "[" + formatIndex(index + 1, "ar-EG", 1) + "] " + label;
        });
        document.getElementById("out").textContent = lines.join("\n");
      </script>
    "#;

    let harness = Harness::from_html(html)?;
    harness.assert_text("#out", "[1] A\n[2] B")?;
    Ok(())
}

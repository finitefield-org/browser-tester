use super::*;

#[test]
fn generator_function_constructor_from_literal_builds_generator_functions() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='out'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const GeneratorFunction = (function* () {}).constructor;
            const byNew = new GeneratorFunction(`
              yield 'a';
              yield 'b';
              yield 'c';
            `);
            const byCall = GeneratorFunction('yield 1; yield* [2, 3];');
            const empty = GeneratorFunction();

            let letters = '';
            for (const value of byNew()) {
              letters += value;
            }
            const numbers = byCall().toArray().join(',');
            const emptyLen = empty().toArray().length;
            const tag = GeneratorFunction.prototype[Symbol.toStringTag];

            document.getElementById('out').textContent =
              letters + '|' + numbers + '|' + emptyLen + '|' + tag;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#out", "abc|1,2,3|0|GeneratorFunction")?;
    Ok(())
}

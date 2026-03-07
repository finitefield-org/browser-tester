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

#[test]
fn global_generator_function_constructor_surface_and_prototype_chain_work() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='out'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const sample = function* namedGen() {};
            const byGlobal = GeneratorFunction('yield 1; yield 2;');
            document.getElementById('out').textContent = [
              typeof GeneratorFunction,
              String(window.GeneratorFunction === GeneratorFunction),
              GeneratorFunction.name,
              String(GeneratorFunction.length),
              String(Object.getPrototypeOf(GeneratorFunction) === Function.prototype),
              String(Object.getPrototypeOf(sample) === GeneratorFunction.prototype),
              byGlobal.name,
              String(byGlobal.length),
              String(byGlobal.prototype.constructor === byGlobal),
              String(byGlobal.toString().includes('__bt_function_ref__(')),
              String(byGlobal().toArray().join(',')),
              String(sample.constructor === GeneratorFunction)
            ].join('|');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#out",
        "function|true|GeneratorFunction|1|true|true|anonymous|0|true|true|1,2|true",
    )?;
    Ok(())
}

#[test]
fn generator_function_native_source_text_and_hidden_surface_work() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='out'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const ctorText = GeneratorFunction.toString();
            const iterProto = GeneratorFunction.prototype.prototype;

            function forInKeys(value) {
              let out = '';
              for (const key in value) {
                out += key + ',';
              }
              return out || 'empty';
            }

            document.getElementById('out').textContent = [
              String(ctorText.includes('[native code]')),
              String(ctorText.includes('GeneratorFunction')),
              String(ctorText === Function.prototype.toString.call(GeneratorFunction)),
              String(ctorText === String(GeneratorFunction)),
              String(ctorText === new String(GeneratorFunction).valueOf()),
              String(Object.keys(GeneratorFunction).length),
              String(Object.keys(GeneratorFunction.prototype).length),
              String(Object.keys(iterProto).length),
              String(Object.keys({ ...GeneratorFunction.prototype }).length),
              String(Object.keys({ ...iterProto }).length),
              forInKeys(GeneratorFunction.prototype),
              forInKeys(iterProto),
              JSON.stringify(iterProto)
            ].join('|');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#out", "true|true|true|true|true|0|0|0|0|0|empty|empty|{}")?;
    Ok(())
}

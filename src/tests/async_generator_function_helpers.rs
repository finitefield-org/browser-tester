use super::*;

#[test]
fn async_generator_function_constructor_builds_async_generator_functions() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='out'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const AsyncGeneratorFunction = (async function* () {}).constructor;
            const byNew = new AsyncGeneratorFunction(`
              yield await Promise.resolve('a');
              yield await Promise.resolve('b');
              yield await Promise.resolve('c');
            `);
            const byCall = AsyncGeneratorFunction(
              'yield await Promise.resolve(1); yield* [2, 3];'
            );
            const empty = AsyncGeneratorFunction();

            const fromNew = byNew();
            const fromCall = byCall();
            const fromEmpty = empty();
            const selfFactory = fromNew[Symbol.asyncIterator];
            const self = selfFactory();
            const tag = AsyncGeneratorFunction.prototype[Symbol.toStringTag];

            Promise.all([
              fromNew.next(),
              fromNew.next(),
              fromNew.next(),
              fromNew.next(),
              fromCall.next(),
              fromCall.next(),
              fromCall.next(),
              fromCall.next(),
              fromEmpty.next(),
            ]).then((results) => {
              const a = results[0];
              const b = results[1];
              const c = results[2];
              const d = results[3];
              const n1 = results[4];
              const n2 = results[5];
              const n3 = results[6];
              const nDone = results[7];
              const emptyDone = results[8];
              const letters = a.value + b.value + c.value;
              const numbers = n1.value + ',' + n2.value + ',' + n3.value;
              const doneFlags =
                d.done + ':' + nDone.done + ':' + emptyDone.done;
              document.getElementById('out').textContent =
                letters + '|' +
                numbers + '|' +
                doneFlags + '|' +
                tag + '|' +
                (self === fromNew) + '|' +
                (typeof fromNew[Symbol.iterator]);
            });
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#out",
        "abc|1,2,3|true:true:true|AsyncGeneratorFunction|true|undefined",
    )?;
    Ok(())
}

#[test]
fn global_async_generator_function_constructor_surface_and_prototype_chain_work() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='out'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const sample = async function* namedAsyncGen() {};
            const byGlobal = AsyncGeneratorFunction(
              'yield await Promise.resolve(1); yield 2;'
            );
            const iter = byGlobal();
            Promise.all([iter.next(), iter.next(), iter.next()]).then((results) => {
              document.getElementById('out').textContent = [
                typeof AsyncGeneratorFunction,
                String(window.AsyncGeneratorFunction === AsyncGeneratorFunction),
                AsyncGeneratorFunction.name,
                String(AsyncGeneratorFunction.length),
                String(Object.getPrototypeOf(AsyncGeneratorFunction) === Function.prototype),
                String(Object.getPrototypeOf(sample) === AsyncGeneratorFunction.prototype),
                byGlobal.name,
                String(byGlobal.length),
                String(byGlobal.constructor === AsyncGeneratorFunction),
                String(byGlobal.toString().includes('__bt_function_ref__(')),
                results[0].value + ',' + results[1].value,
                String(results[2].done)
              ].join('|');
            });
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#out",
        "function|true|AsyncGeneratorFunction|1|true|true|anonymous|0|true|true|1,2|true",
    )?;
    Ok(())
}

#[test]
fn async_generator_function_native_source_text_and_hidden_surface_work() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='out'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const ctorText = AsyncGeneratorFunction.toString();
            const iterProto = AsyncGeneratorFunction.prototype.prototype;

            function forInKeys(value) {
              let out = '';
              for (const key in value) {
                out += key + ',';
              }
              return out || 'empty';
            }

            document.getElementById('out').textContent = [
              String(ctorText.includes('[native code]')),
              String(ctorText.includes('AsyncGeneratorFunction')),
              String(ctorText === Function.prototype.toString.call(AsyncGeneratorFunction)),
              String(ctorText === String(AsyncGeneratorFunction)),
              String(ctorText === new String(AsyncGeneratorFunction).valueOf()),
              String(Object.keys(AsyncGeneratorFunction).length),
              String(Object.keys(AsyncGeneratorFunction.prototype).length),
              String(Object.keys(iterProto).length),
              String(Object.keys({ ...AsyncGeneratorFunction.prototype }).length),
              String(Object.keys({ ...iterProto }).length),
              forInKeys(AsyncGeneratorFunction.prototype),
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

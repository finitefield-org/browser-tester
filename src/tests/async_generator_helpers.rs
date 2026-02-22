use super::*;

#[test]
fn async_generator_instances_support_protocol_tags_and_control_methods() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='out'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const AsyncGeneratorFunction = (async function* () {}).constructor;
            const AsyncGeneratorPrototype = AsyncGeneratorFunction.prototype.prototype;

            const iter = (async function* () {
              yield Promise.resolve(1);
              yield 2;
            })();
            const selfFactory = iter[Symbol.asyncIterator];
            const self = selfFactory();
            const constructorTag = iter.constructor[Symbol.toStringTag];
            const generatorTag = iter[Symbol.toStringTag];
            const prototypeConstructorMatches =
              AsyncGeneratorPrototype.constructor === AsyncGeneratorFunction.prototype;

            const firstPromise = iter.next();
            const returnPromise = iter.return(Promise.resolve(99));
            const afterReturnPromise = iter.next();

            const thrower = (async function* () {
              yield 'x';
              yield 'y';
            })();
            const thrownFirstPromise = thrower.next();
            const thrownPromise = thrower.throw('boom');
            const thrownReasonPromise = thrownPromise.then(
              () => 'noerr',
              (reason) => reason
            );
            const afterThrowPromise = thrower.next();

            Promise.all([
              firstPromise,
              returnPromise,
              afterReturnPromise,
              thrownFirstPromise,
              thrownReasonPromise,
              afterThrowPromise,
            ]).then((results) => {
              const first = results[0];
              const returned = results[1];
              const afterReturn = results[2];
              const thrownFirst = results[3];
              const thrownReason = results[4];
              const afterThrow = results[5];

              document.getElementById('out').textContent =
                first.value + ':' + first.done + '|' +
                returned.value + ':' + returned.done + ':' + afterReturn.done + '|' +
                thrownFirst.value + ':' + thrownReason + ':' + afterThrow.done + '|' +
                constructorTag + ':' + generatorTag + '|' +
                prototypeConstructorMatches + ':' + (self === iter) + '|' +
                (typeof firstPromise) + ':' +
                (typeof returnPromise) + ':' +
                (typeof thrownPromise);
            });
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#out",
        "1:false|99:true:true|x:boom:true|AsyncGeneratorFunction:AsyncGenerator|true:true|object:object:object",
    )?;
    Ok(())
}

#[test]
fn async_generator_iteration_resolves_delayed_promises() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='out'></p>
        <script>
          const delayedValue = function(time, value) {
            return new Promise((resolve) => {
              setTimeout(() => resolve(value), time);
            });
          };

          const generate = async function* () {
            yield delayedValue(20, 1);
            yield delayedValue(10, 2);
            yield delayedValue(5, 3);
            yield delayedValue(3, 4);
            yield delayedValue(2, 5);
            yield delayedValue(1, 6);
          };

          document.getElementById('run').addEventListener('click', () => {
            const generator = generate();
            Promise.all([
              generator.next(),
              generator.next(),
              generator.next(),
              generator.next(),
              generator.next(),
              generator.next(),
              generator.next(),
            ]).then((results) => {
              const values = results
                .slice(0, 6)
                .map((result) => result.value)
                .join(',');
              const done = results[6].done;
              document.getElementById('out').textContent = values + '|' + done;
            });
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#out", "")?;
    h.flush()?;
    h.assert_text("#out", "1,2,3,4,5,6|true")?;
    Ok(())
}

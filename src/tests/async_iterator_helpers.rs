use super::*;

#[test]
fn blob_stream_exposes_async_iterator_and_promise_based_next() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='out'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const stream = new Blob(['AB']).stream();
            const iteratorFactory = stream[Symbol.asyncIterator];
            const iterator = iteratorFactory();
            const selfFactory = iterator[Symbol.asyncIterator];
            const self = selfFactory();

            Promise.all([iterator.next(), iterator.next()]).then((results) => {
              const first = results[0];
              const second = results[1];
              document.getElementById('out').textContent =
                (typeof iteratorFactory) + ':' +
                (self === iterator) + ':' +
                first.value.join(',') + ':' +
                first.done + ':' +
                second.done + ':' +
                (typeof iterator[Symbol.iterator]);
            });
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#out", "object:true:65,66:false:true:undefined")?;
    Ok(())
}

#[test]
fn async_iterator_async_dispose_calls_return_when_present() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='out'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const stream = new Blob(['x']).stream();
            const iteratorFactory = stream[Symbol.asyncIterator];
            const withReturn = iteratorFactory();
            let calls = 0;
            withReturn.return = () => {
              calls += 1;
              return Promise.resolve('closed');
            };

            const withoutReturn = iteratorFactory();
            const disposeWithReturnFn = withReturn[Symbol.asyncDispose];
            const disposeWithoutReturnFn = withoutReturn[Symbol.asyncDispose];
            const disposeWithReturn = disposeWithReturnFn();
            const disposeWithoutReturn = disposeWithoutReturnFn();

            Promise.all([disposeWithReturn, disposeWithoutReturn]).then((values) => {
              document.getElementById('out').textContent =
                calls + ':' +
                (values[1] === undefined) + ':' +
                (typeof disposeWithReturnFn);
            });
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#out", "1:true:object")?;
    Ok(())
}

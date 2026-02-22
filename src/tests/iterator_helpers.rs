use super::*;

#[test]
fn iterator_from_map_filter_take_and_to_array_work() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='out'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const values = Iterator.from([1, 2, 3, 4, 5])
              .filter((value) => value % 2 === 1)
              .map((value, index) => value + index)
              .take(2)
              .toArray();
            document.getElementById('out').textContent = values.join(',');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#out", "1,4")?;
    Ok(())
}

#[test]
fn iterator_reduce_find_some_every_and_foreach_work() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='out'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const sum = Iterator.from([1, 2, 3]).reduce((acc, value) => acc + value, 0);
            const sumNoInitial = Iterator.from([5, 6]).reduce((acc, value) => acc + value);
            const found = Iterator.from([1, 3, 8, 9]).find((value) => value % 2 === 0);
            const some = Iterator.from([1, 3, 8]).some((value) => value > 7);
            const every = Iterator.from([2, 4, 6]).every((value) => value % 2 === 0);
            let total = 0;
            Iterator.from([1, 2, 3]).forEach((value) => {
              total += value;
            });
            document.getElementById('out').textContent =
              sum + ':' + sumNoInitial + ':' + found + ':' + some + ':' + every + ':' + total;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#out", "6:11:8:true:true:6")?;
    Ok(())
}

#[test]
fn iterator_concat_next_and_for_of_work() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='out'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const iter = Iterator.concat([1, 2], new Set([3, 4]), '56');
            const first = iter.next();
            let tail = '';
            for (const value of iter) {
              tail = tail + value;
            }
            document.getElementById('out').textContent =
              first.value + ':' + first.done + ':' + tail;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#out", "1:false:23456")?;
    Ok(())
}

#[test]
fn array_values_returns_iterator_and_iterator_from_reuses_iterator_input() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='out'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const arrIterator = [10, 20, 30].values();
            const first = arrIterator.next();
            const rest = arrIterator.toArray();

            const base = Iterator.from([1, 2, 3]);
            const same = Iterator.from(base);
            const a = base.next().value;
            const b = same.next().value;
            const c = base.next().value;

            document.getElementById('out').textContent =
              first.value + ':' + first.done + ':' + rest.join(',') + '|' + a + ':' + b + ':' + c;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#out", "10:false:20,30|1:2:3")?;
    Ok(())
}

# browser-tester

Rustだけで実装された、決定論的なブラウザライクテスト用クレートです。

- 設計ドキュメント: `doc/e2e-lite-runtime-design.md`

## 目的

- DOM とスクリプトのテストを、同一プロセス内で決定論的に実行できるランタイムを提供すること。
- 外部ブラウザ、WebDriver、Node.js に依存せずにブラウザ操作テストを実現すること。

## 使い方

1. HTML からテストハーネスを作成します。
2. セレクタを使って要素を操作します。
3. 期待する DOM 状態をアサートします。

```rust
use browser_tester::Harness;

fn main() -> browser_tester::Result<()> {
    let html = r#"
      <input id='name' />
      <button id='submit'>Submit</button>
      <p id='result'></p>
      <script>
        document.getElementById('submit').addEventListener('click', () => {
          const name = document.getElementById('name').value;
          document.getElementById('result').textContent = `Hello, ${name}`;
        });
      </script>
    "#;

    let mut harness = Harness::from_html(html)?;
    harness.type_text("#name", "Alice")?;
    harness.click("#submit")?;
    harness.assert_text("#result", "Hello, Alice")?;
    Ok(())
}
```

テスト実行:

```bash
cargo test
```

## ランタイム方針

- セキュリティと決定論を維持するため、`eval` は意図的に実装していません。
- 時刻 API は fake clock を基準にしており、`Date.now()` と `performance.now()` を提供します。

## テストモック

- `fetch` はテスト時にモックへ差し替える前提で設計されています。
- `confirm` / `prompt` は、戻り値をモック注入できる API を提供します。
- 主な API:
  - `Harness::set_fetch_mock(url, body)`
  - `Harness::enqueue_confirm_response(bool)`
  - `Harness::enqueue_prompt_response(Option<&str>)`

開発: [Finite Field, K.K.](https://finitefield.org)

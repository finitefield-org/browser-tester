# browser-tester

- Design doc: `doc/e2e-lite-runtime-design.md`

## Runtime Policy

- `eval` は実装しません。セキュリティと決定論維持のため、動的コード実行は非対応方針です。
- 時刻APIは `Date.now()` を使用し、`performance.now()` は実装対象外です。

## Test Mocks

- `fetch` はテスト用途向けにモック差し替え前提で実装します。
- `confirm` / `prompt` は返り値をモック注入できるAPIを提供します。
- 代表API:
  - `Harness::set_fetch_mock(url, body)`
  - `Harness::enqueue_confirm_response(bool)`
  - `Harness::enqueue_prompt_response(Option<&str>)`

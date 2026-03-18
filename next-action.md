# browser-tester 現行コードへの適用可能な次アクション

作成日: 2026-03-18

この文書は、[`next.md`](./next.md) の振り返りを「現状のコードベースに今から適用できるか」という観点で整理したものである。

結論から言うと、全面作り直し前提の話もある一方で、現行版にすぐ適用できる改善はかなりある。
特に効果が大きいのは、次の 4 系統である。

1. 公開 surface の整理
2. 文書の分離と同期コスト削減
3. 巨大ファイルと責務集中の分解
4. テストの役割分離

---

## 1. 全体判断

`next.md` の内容を現行コードに当てはめると、次のように分かれる。

### 1.1 今すぐ適用できる

- README の縮小と文書分離
- capability matrix の導入
- `Harness` 公開 API のカテゴリ整理
- mock 追加時の運用ルール固定
- 巨大ファイルの分割開始
- public contract tests の独立

### 1.2 段階的に適用できる

- `Harness` の責務を facade 寄りに戻す
- script runtime 系のモジュール境界を強める
- test taxonomy を導入して既存テストを再配置する
- docs と tests を公開機能単位で対応づける

### 1.3 現行コードに無理やり当てるべきではない

- workspace への全面分割
- DOM データモデルの全面再設計
- Script runtime の全面作り直し
- capability 単位での公開 API 再編成を大きな breaking change として一気に入れること

つまり、今のコードでやるべきなのは「全面改造」ではなく、「現行版の保守コストを下げるための整理」と「次回設計への移行路を先に作ること」である。

---

## 2. いま最も適用価値が高い項目

## 2.1 README を短くして、設計文書を分離する

### なぜ今やる価値が高いか

現状の `README.md` は 1,160 行あり、使い方、mock、詳細設計、低レベル設計が混在している。
これは保守コストが高く、機能追加時の同期漏れも起こしやすい。

### 現行コードにそのまま適用できる理由

これはコードの挙動を変えずに適用できる。
しかも、今後の mock 追加や API 拡張のたびに効く。

### 具体アクション

- `README.md` は quick start と主要制約に絞る
- 設計の大部分は `doc/architecture.md` に移す
- mock の使い方は `doc/mock-guide.md` に移す
- `README.md` からそれらにリンクする
- `README` 内の「設計書と同期している」という記述は、実在ファイルだけを参照する形に直す

### 期待効果

- 利用者が読みやすくなる
- 設計更新の心理的コストが下がる
- mock 追加時の README 更新範囲が小さくなる

---

## 2.2 capability matrix を追加して、公開 surface を分類する

### なぜ今やる価値が高いか

現行版は `Harness` の公開面が広いが、どこまでを「安定契約」とみなすかが明文化されていない。
この状態だと、修正時に何を絶対に壊してはいけないかが曖昧になる。

### 現行コードにそのまま適用できる理由

挙動は変えず、文書とテスト分類だけで始められる。
しかも、今後の実装判断の基準になる。

### 具体アクション

`doc/capability-matrix.md` を追加し、少なくとも次を分類する。

- Stable Core
  - `from_html`
  - `click`
  - `type_text`
  - `set_checked`
  - `submit`
  - `advance_time`
  - `assert_*`
- Stable Test Mocks
  - `set_fetch_mock`
  - `set_clipboard_text`
  - `set_location_mock_page`
  - `set_input_files`
- Extended / Browser-like
  - `location` / `history`
  - `download artifacts`
  - `matchMedia`
  - `cacheStorage`
  - `cookieStore`
  - `canvas` / `media` 周辺
- Internal Only
  - runtime helper 群
  - object/value helper 群

### 関連箇所

- `src/harness_api.rs`
- `src/core_impl/runtime/runtime_platform/bootstrap/environment_global_init.rs`
- `src/core_impl/runtime/runtime_platform/dom_actions/user_actions_forms.rs`
- `src/core_impl/runtime/runtime_platform/dom_actions/trace_mocks_input_primitives.rs`
- `src/core_impl/runtime/runtime_platform/dom_actions/timer_controls_execution.rs`
- `src/core_impl/runtime/runtime_platform/dom_actions/assertions_form_helpers.rs`

### 期待効果

- 公開 API の安定性を判断しやすくなる
- regression test と contract test の線引きがしやすくなる
- 次に足すべき機能の優先順位が見えやすくなる

---

## 2.3 `Harness` API をカテゴリ単位で見える化する

### なぜ今やる価値が高いか

`Harness` の公開 API は利用者から見ると素直だが、実装は複数ファイルに分散している。
そのため、公開 surface の全体像が把握しづらい。

### 現行コードにそのまま適用できる理由

これは API を壊さずにできる。
まずは rustdoc と文書上の整理だけでも効果がある。

### 具体アクション

- 公開 API を次のカテゴリで整理する
  - Constructors
  - Actions
  - Assertions
  - Time / Scheduler
  - Mocks
  - Trace / Debug
- `README.md` と `doc/capability-matrix.md` も同じカテゴリ構成に揃える
- 将来的に `mocks_mut()` のような subview を導入するなら、その前段としてカテゴリ設計を固定する

### 期待効果

- `Harness` はそのままでも、神オブジェクト化の認知負荷を下げられる
- API の追加先を決めやすくなる

---

## 2.4 mock 追加時のルールを現行運用に固定する

### なぜ今やる価値が高いか

現行版の強みの一つは mock だが、強い機能ほど運用ルールがないと README と実装がずれやすい。

### 現行コードにそのまま適用できる理由

これは将来の作法を決めるだけで、今すぐ始められる。

### 具体アクション

新しい test-only mock を追加するときは、必ず次をセットで入れる。

- 公開 API
- 最小使用例
- 失敗系のテスト
- call capture または artifact capture の説明
- `README.md` 更新
- `doc/mock-guide.md` 更新

このルールは `AGENTS.md` か将来の `CONTRIBUTING.md` に書く価値がある。

### 期待効果

- mock の使い方が属人化しにくくなる
- 後から使い方を探すコストが減る

---

## 3. 巨大ファイル分割は、現行コードでも十分やる価値がある

これは `next.md` の中でも、現行コードへもっとも直接効く項目である。

### 3.1 優先度が高いホットスポット

現状の大きな集中点は次の通り。

- `src/core_impl/runtime/runtime_exec/member_calls_ops/value_object_helpers.rs`: 11,682 行
- `src/core_impl/runtime/runtime_platform/script_runtime/callable_execution.rs`: 8,613 行
- `src/core_impl/runtime/runtime_platform/script_runtime/statement_execution.rs`: 6,617 行
- `src/core_impl/runtime/runtime_platform/dom_actions/user_actions_forms.rs`: 1,286 行
- `src/script_ast.rs`: 1,427 行

この規模になると、「挙動を変えない整理」だけでも効果が大きい。

### 3.2 今のコードに適用しやすい分割方針

#### `value_object_helpers.rs`

一気に設計変更するのではなく、まず責務ごとに分ける。

- object property lookup
- property descriptor / writable / enumerable / configurable
- wrapper object helpers
- callable metadata
- internal key helpers

#### `callable_execution.rs`

- call setup
- constructor path
- lexical environment capture
- callback invocation
- closure / bound callable 補助

#### `statement_execution.rs`

- declaration / assignment
- control flow
- exception handling
- loop execution
- function / class / scope 補助

#### `user_actions_forms.rs`

- text/select actions
- check/radio actions
- click activation
- focus/blur/copy/paste/cut
- submit / requestSubmit 系

### 3.3 分割時の注意

- まずはファイル分割だけを目的にする
- 挙動変更を同時に入れない
- module root を薄い façade にして、既存呼び出し側を極力変えない
- regression test を先に固定する

### 期待効果

- レビューしやすくなる
- 今後のバグ修正が局所化しやすくなる
- 次回の大きな設計変更に向けた足場になる

---

## 4. テストの役割分離は、今から始められる

現行版のテスト資産は厚い。
ただし、厚いことと整理されていることは別である。

### 4.1 いまの課題

- `src/tests/mod.rs` に非常に多くのモジュールが並ぶ
- API テスト、仕様テスト、バグ回帰テスト、巨大回帰群が混在している
- `tests/integration_cases/mod.rs` も issue 系中心で、public contract が見えにくい

### 4.2 現行コードに適用できる改善

まずは大移動ではなく、public contract test を新設する。

### 4.3 具体アクション

- `tests/contract_harness_core.rs` を追加する
- そこに次だけをまず置く
  - `from_html`
  - `click`
  - `type_text`
  - `set_checked`
  - `submit`
  - `advance_time`
  - `assert_*`
  - 代表的 mock (`fetch`, `clipboard`, `location`, `file input`)
- 既存の `src/tests` は当面そのままにして、今後の新規 public contract だけは新しい置き場へ寄せる

### 4.4 このやり方がよい理由

- 既存テストを壊さず始められる
- 「これは絶対守る public contract」という核を先に作れる
- 大規模整理の前に価値が出る

---

## 5. 現行コードに適用できる設計ルール

次回設計のうち、以下は今のコードにも導入できる。

### 5.1 ファイルサイズガード

- 目安として 800 行超で分割検討
- 1,500 行超は原則として要分割計画

### 5.2 追加先の先決め

新機能追加時は、先に「これはどの subsystem の責務か」を決める。

現行コードなら最低でも次を意識する。

- DOM
- parser
- script runtime
- event / user actions
- timer / scheduler
- mocks / trace

### 5.3 公開 API 追加時の必須項目

- capability matrix 更新
- README の該当箇所更新
- public contract test 追加
- regression test 追加

### 5.4 「便利だから `Harness` に足す」を抑える

今後の追加 API は、最低限次を確認してから入れる。

- 本当に公開 API か
- 既存 API の組み合わせで足りないか
- test-only mock に分類すべきではないか
- debug / trace 用 API として分けるべきではないか

---

## 6. いまは適用しない方がよい項目

以下は `next.md` には重要だが、現行コードへ直ちに当てるには重すぎる。

### 6.1 workspace への全面分割

理由:

- import 経路が広く、局所整理では済まない
- 今のフェーズではリスクが高い

判断:

- いまはやらない
- ただし、ファイル分割と責務整理で布石は打つ

### 6.2 DOM データモデルの全面刷新

理由:

- `NodeId`、indexes、runtime state の広範囲に影響する
- これは現行保守ではなく再設計案件

判断:

- 次回設計向け
- 現行版では局所改善に留める

### 6.3 Script runtime の全面再設計

理由:

- parser / evaluator / runtime state / value model が深く結合している
- 現行版に途中から入れると、整理より破壊の方が大きい

判断:

- 今は巨大ファイル分割と責務の明確化に留める

---

## 7. おすすめ実行順

現行版に適用するなら、順番は次がよい。

1. `README` 分離
2. `doc/capability-matrix.md` 追加
3. public contract tests 新設
4. mock 追加ルールを文書化
5. `user_actions_forms.rs` 分割
6. `trace_mocks_input_primitives.rs` を mock family ごとに整理
7. `value_object_helpers.rs` / `callable_execution.rs` / `statement_execution.rs` を順に分割

この順番なら、挙動変更の少ないところから先に始められ、後半の大きい整理にも入りやすい。

---

## 8. 最終結論

`next.md` の振り返りは、現行コードにも十分適用できる。
ただし、適用の仕方は「理想設計を一気に入れる」ではない。

現行版でやるべきなのは、

- 文書と公開 surface の整理
- mock 運用ルールの固定
- 巨大ファイルの責務分割
- public contract tests の独立

である。

逆に、

- workspace 化
- DOM モデル刷新
- script runtime 再設計

は次回の大きな設計変更向けに温存すべきである。

つまり次の一手は、全面作り直しではなく、
「現行版の保守性を上げながら、次回設計への移行路を作ること」
になる。

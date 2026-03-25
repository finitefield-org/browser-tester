# Workspace Instructions

このリポジトリでは browser-tester の Rust workspace を正とする。設計判断は root 配下の文書を基準にすること。

必要に応じ、テスト用途に API をモックできる作りにしてください。
その場合、README.md と doc/mock-guide.md にそのモックの使い方を記載してください。
HTML の仕様は html-standard フォルダにあるので適宜参照してください。legacy な仕様や deprecated な仕様は実装する必要ありません。

## 最初に参照する文書

実装を始める前に、少なくとも次を確認すること。

1. README.md
2. doc/implementation-guide.md
3. doc/subsystem-map.md
4. doc/capability-matrix.md
5. doc/roadmap.md

必要に応じて追加で参照する文書:

- doc/architecture.md
- doc/mock-guide.md
- doc/limitations.md
- doc/adr/*.md

## 実装時の基本方針

- 一度に広く作らない。1 回の変更で 1 capability を前進させる。
- 先に owning subsystem を決めてから実装する。
- 実装より先に、どの test layer で固定するかを決める。
- `Harness` は thin facade のまま保つ。
- 高度な設定や test-only 機能は、まず subview や mock registry を検討する。
- 部分実装のまま曖昧に通さない。未対応は明示エラーにする。

## 置き場の判断

公開 API や内部コードの置き場は、必ず `doc/subsystem-map.md` を見て決めること。

大まかな責務:

- `crates/browser-tester/`: public facade, public error taxonomy, public views
- `crates/bt-dom/`: DOM, HTML parsing, selector, indexes, side tables
- `crates/bt-runtime/`: Session, scheduler, services, mocks, debug state
- `crates/bt-script/`: script parser/evaluator, host bindings

## 進める順序

日々の実装順は `doc/implementation-guide.md` を基準にすること。

基本順:

1. DOM bootstrap
2. selector subset
3. read-only assertions
4. script minimum slice
5. event dispatch
6. forms and user actions
7. mock integration
8. hardening

`click` や `type_text` を先に増やすのは禁止。
DOM と selector の土台ができる前に user action を増やさないこと。

## 公開 API を追加・変更するときの注意

新しい公開 API、特に `Harness` のメソッドを追加または変更するときは、必ず次を確認すること。

- それは本当に公開 API か
- 既存 API の組み合わせで足りないか
- test-only mock に分類すべきではないか
- debug / trace 用 API として分けるべきではないか
- `doc/capability-matrix.md` を更新したか
- `README.md` を更新したか
- public contract test を追加または更新したか
- regression test を追加または更新したか
- failure-path test を追加したか

公開 API は、実装完成前でも silent fallback を持たせないこと。
未完成なら明示的に unsupported error を返すこと。

## test-only mock を追加・変更するときの注意

必要に応じ、テスト用途に API をモックできる作りにしてよい。
その場合、mock は escape hatch ではなく正式な test API として扱うこと。

新しい test-only mock を追加するときは、必ず次をセットで行うこと。

- 公開 API の追加または更新
- 最小使用例の追加
- 失敗系を含むテストの追加
- call capture または artifact capture の説明追加
- README.md の更新
- doc/mock-guide.md の更新
- doc/capability-matrix.md の更新

mock family を増やす前に、typed registry 配下に置けないかを先に検討すること。
`Harness` 直下に `set_*` を増殖させないこと。

## テスト方針

初期フェーズでは次を優先すること。

- subsystem tests
- 小さい public contract tests
- failure-path tests

次は後回しにすること。

- capability が安定する前の大きい regression fixture
- public contract が決まる前の browser comparison

1 つの capability を public にするなら、最低でも次を揃えること。

- public contract test
- owning subsystem の test
- failure-path test
- 対応する文書更新

## 文書更新ルール

以下は実装と同じ変更で更新すること。

- support level や公開保証が変わったら `doc/capability-matrix.md`
- mock の入口や capture が増えたら `doc/mock-guide.md`
- 実装順や進め方の判断が変わったら `doc/implementation-guide.md`
- 所有権や置き場の方針が変わったら `doc/subsystem-map.md`
- 利用者向け入口が変わったら `README.md`

## この workspace 固有の注意

- 既存実装とテストから得られた知見は再利用してよい。
- 既存クレートの巨大な責務集中を再発させないこと。
- script runtime 固有型を `bt-dom` や public facade に漏らさないこと。
- binding 実装から DOM 内部構造へ直接書き込まないこと。
- 依存追加は慎重に行い、挙動の中核を外部実装に委ねないこと。

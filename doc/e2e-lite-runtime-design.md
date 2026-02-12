# Lightweight HTML+JS Test Runtime Design (Rust)

## 1. 背景と目的

`chromedp` のような実ブラウザ起動型E2Eは、起動・描画・ネットワーク・プロセス間通信のオーバーヘッドが大きく、フィードバックが遅くなりやすい。
本設計は、**単一HTMLファイル（インラインJSのみ）を対象**に、DOMとイベントの振る舞いだけを高速に検証するためのRust製テストランタイムを定義する。

主目的:
- フォーム入力、チェックボックス操作、ボタン押下、結果テキスト検証を高速に実行
- テストはRustのユニットテストとして実行可能
- 実ブラウザ互換を100%目指さず、対象業務に必要な範囲で安定実装する

## 2. スコープ

### 2.1 In Scope
- 1枚のHTML文字列を読み込み、DOMを構築
- インライン`<script>`実行
- DOM操作 (`querySelector`, `getElementById`, `textContent`, `value`, `checked`)
- イベントシステム (`click`, `input`, `change`, `submit`)
- キャプチャ/バブル、`preventDefault`, `stopPropagation`
- テストハーネスAPI（操作 + アサート）
- 失敗時の差分表示

### 2.2 Out of Scope
- 外部CSS/JSファイルの読み込み
- AJAX/fetch/XHR/WebSocket
- 画面描画、レイアウト計算、スタイル適用、アクセシビリティツリー
- iframe、shadow DOM、custom elements（MVPでは非対応）

## 3. 要件

### 3.1 機能要件
1. `Harness::from_html` でHTML初期化できる
2. `type_text`, `set_checked`, `click`, `submit` が呼べる
3. JSのイベントハンドラによりDOMが更新される
4. `assert_text`, `assert_value`, `assert_checked`, `assert_exists` ができる
5. 失敗時にセレクタ対象の実値と期待値を明示する

### 3.2 非機能要件
- 単体テスト1ケースあたり数ms〜数十msを目標（HTML規模依存）
- テスト間の完全独立（状態リーク防止）
- 決定論的な実行（時刻・乱数・非同期を固定可能）

## 4. 全体アーキテクチャ

```mermaid
flowchart LR
  T["Rust Test (cargo test)"] --> H["test_harness"]
  H --> R["runtime_core"]
  R --> D["dom_core"]
  R --> E["event_system"]
  R --> J["js_runtime (Boa)"]
  D <--> J
  E <--> D
```

モジュール:
- `dom_core`: DOM木、セレクタ、属性/プロパティ
- `js_runtime`: JSエンジン統合とWeb APIバインディング
- `event_system`: イベント伝播と既定動作
- `runtime_core`: 初期化、スクリプト実行、タスクキュー
- `test_harness`: 高水準のテスト操作API

## 5. クレート構成（推奨）

```
crates/
  runtime-core/
    src/lib.rs
    src/runtime.rs
    src/task_queue.rs
  dom-core/
    src/lib.rs
    src/node.rs
    src/document.rs
    src/query.rs
  event-system/
    src/lib.rs
    src/event.rs
    src/dispatch.rs
  js-runtime/
    src/lib.rs
    src/boa_bindings.rs
    src/web_api/*.rs
  test-harness/
    src/lib.rs
    src/actions.rs
    src/assertions.rs
```

最初は単一クレートでも良いが、将来的な境界明確化のため上記分割を推奨。

## 6. DOMモデル詳細

### 6.1 データ構造
- Arena方式（`Vec<Node>`） + `NodeId(usize)`
- 各ノード:
  - `node_type`: Document / Element / Text
  - `parent: Option<NodeId>`
  - `children: Vec<NodeId>`
  - `tag_name`（Elementのみ）
  - `attributes: HashMap<String, String>`
  - `properties: ElementProperties`

`ElementProperties` (MVP):
- `value: String`（input/textarea）
- `checked: bool`（checkbox/radio）
- `disabled: bool`
- `selected_index: Option<usize>`（selectを見据えた拡張）

### 6.2 インデックス
- `id_index: HashMap<String, NodeId>`
- `class_index: HashMap<String, Vec<NodeId>>`（必要時）
- 先に `id` と単純セレクタ最適化のみ実装で十分

### 6.3 セレクタ
MVP対応:
- `#id`, `.class`, `tag`, `tag.class`, `[name=value]`
- 子孫結合子（空白）

非対応セレクタは明示的エラーにする（サイレント無視しない）。

## 7. JSランタイム詳細

### 7.1 エンジン選定
- 第一候補: `boa_engine`（純Rust、配布容易）
- 代替: `rquickjs`（互換性高めだが依存増）

本設計は `boa_engine` を前提に記載。

### 7.2 グローバルオブジェクト
JSへ公開する最小API:
- `window`
- `document`
  - `getElementById`
  - `querySelector`, `querySelectorAll`
  - `createElement`（将来拡張）
- `Element`
  - `textContent` getter/setter
  - `getAttribute`, `setAttribute`
  - `addEventListener`, `removeEventListener`, `dispatchEvent`
- `HTMLInputElement`
  - `value`, `checked`

### 7.3 Rust<->JSブリッジ
- JS objectに`NodeId`を内部フィールドとして保持
- メソッド呼び出し時に `NodeId` をRust側へ渡してDOM操作
- DOM更新後は必要に応じてid index等を同期

## 8. イベントシステム詳細

### 8.1 Eventオブジェクト
フィールド:
- `type`, `target`, `currentTarget`, `bubbles`, `cancelable`, `defaultPrevented`
- 内部制御: `propagation_stopped`, `immediate_propagation_stopped`

### 8.2 伝播アルゴリズム
1. `target`からrootまでのpath構築
2. Captureフェーズ（root -> parent of target）
3. Targetフェーズ（target）
4. Bubbleフェーズ（parent of target -> root）

`stopPropagation` は以降フェーズ停止、`stopImmediatePropagation` は同一ノード残りリスナーも停止。

### 8.3 既定動作（重要）
`click` on checkbox:
1. `checked`トグル
2. `input`発火
3. `change`発火

`click` on submit button:
1. 祖先`form`の`submit`イベント発火
2. `preventDefault`されなければ成功扱い（遷移は実装しない）

## 9. Runtime実行モデル

### 9.1 初期化
1. HTML parse (`html5ever`)
2. DOM構築
3. `<script>`を文書順で同期実行
4. 初期タスクキュー実行

### 9.2 タスクキュー
- MVPは同期実行が基本
- 将来のため microtask風キューを保持
- `harness.flush()` で明示排出可能

### 9.3 決定論サポート
- `Date.now`, `Math.random` を固定値注入可能にする
- テストごとにseed指定

## 10. テストハーネスAPI詳細

```rust
pub struct Harness { /* runtime */ }

impl Harness {
    pub fn from_html(html: &str) -> Result<Self, Error>;

    // Action
    pub fn type_text(&mut self, selector: &str, text: &str) -> Result<()>;
    pub fn set_checked(&mut self, selector: &str, checked: bool) -> Result<()>;
    pub fn click(&mut self, selector: &str) -> Result<()>;
    pub fn submit(&mut self, selector: &str) -> Result<()>;
    pub fn dispatch(&mut self, selector: &str, event: &str) -> Result<()>;
    pub fn flush(&mut self) -> Result<()>;

    // Assert
    pub fn assert_text(&self, selector: &str, expected: &str) -> Result<()>;
    pub fn assert_value(&self, selector: &str, expected: &str) -> Result<()>;
    pub fn assert_checked(&self, selector: &str, expected: bool) -> Result<()>;
    pub fn assert_exists(&self, selector: &str) -> Result<()>;
}
```

### 10.1 Actionの内部仕様
- `type_text`:
  - 対象`value`を置換
  - `input`イベント発火
- `set_checked`:
  - 既存値と異なるときのみ更新
  - `input` -> `change`
- `click`:
  - `click`イベント発火
  - 要素型に応じ既定動作実施

## 11. エラー設計

`Error`分類:
- `SelectorNotFound { selector }`
- `UnsupportedSelector { selector }`
- `TypeMismatch { selector, expected, actual }`
- `JsException { message, stack }`
- `AssertionFailed { selector, expected, actual, dom_snippet }`

失敗時は次を必ず含める:
- 対象セレクタ
- 期待値/実値
- 対象ノード周辺のHTML断片（最大N文字）

## 12. ログ・デバッグ

- `Harness::enable_trace(true)` でイベントトレース有効化
- 出力例:
  - `[event] click target=#submit phase=target`
  - `[event] submit target=form#signup phase=bubble defaultPrevented=false`

- `dump_dom(selector)` で部分DOMを文字列化

## 13. テスト戦略

### 13.1 仕様テスト（ランタイム向け）
- イベント順序テスト
- `stopPropagation`挙動
- checkbox既定動作
- `preventDefault`時のsubmit抑止

### 13.2 利用者向けサンプルテスト
- 入力 + チェック + ボタン押下 + 結果文言確認
- バリデーション失敗時メッセージ確認

### 13.3 回帰テスト運用
- 過去バグは必ずfixture HTML化
- fixtureごとに期待スナップショット保有

## 14. 実装フェーズ

### Phase 1 (MVP)
- DOM基本操作
- `querySelector`最小対応
- インラインscript実行
- `click/input/change` + assert

### Phase 2
- `submit`、フォーム要素拡張
- エラー/差分表示強化
- トレースログ

### Phase 3
- セレクタ拡張
- microtaskの安定化
- 高速化（index、再利用）

## 15. 代表的ユースケース

```rust
#[test]
fn submit_updates_result() -> anyhow::Result<()> {
    let html = r#"
    <input id='name'>
    <input id='agree' type='checkbox'>
    <button id='submit'>Send</button>
    <p id='result'></p>
    <script>
      document.getElementById('submit').addEventListener('click', () => {
        const name = document.getElementById('name').value;
        const agree = document.getElementById('agree').checked;
        document.getElementById('result').textContent =
          agree ? `OK:${name}` : 'NG';
      });
    </script>
    "#;

    let mut h = Harness::from_html(html)?;
    h.type_text("#name", "Taro")?;
    h.set_checked("#agree", true)?;
    h.click("#submit")?;
    h.assert_text("#result", "OK:Taro")?;
    Ok(())
}
```

## 16. 技術選定

推奨クレート:
- HTML parse: `html5ever`
- Selector: `selectors`（または`kuchiki`連携）
- JS engine: `boa_engine`
- Error: `thiserror`
- Snapshot test: `insta`

## 17. 既知リスクと対策

1. JS互換性不足（ES機能差）
- 対策: 対象HTMLのJS制約を定義し、非対応構文を早期エラー化

2. DOM仕様の実装漏れ
- 対策: 必須Web API一覧を契約化して段階実装

3. イベント順序のズレ
- 対策: 仕様テストを先に固定し、変更時にCIで検出

## 18. 受け入れ基準（DoD）

1. 単一HTML fixtureで主要ユースケースが3件以上通る
2. 失敗時ログにセレクタ・期待値・実値・DOM断片が出る
3. `cargo test`で安定再現（連続実行でflakyなし）
4. 新規fixture追加が容易（10分以内で1ケース追加可能）

---

この設計は、ブラウザ完全互換ではなく、**フォーム中心UIのロジック検証を最短で高速化するための実用設計**として定義している。

## 19. 低レベル実装設計

### 19.1 主要型定義（案）

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct NodeId(pub usize);

#[derive(Debug)]
pub enum NodeType {
    Document,
    Element(ElementData),
    Text(String),
}

#[derive(Debug)]
pub struct ElementData {
    pub tag_name: String,
    pub attributes: std::collections::HashMap<String, String>,
    pub props: ElementProps,
}

#[derive(Debug, Default)]
pub struct ElementProps {
    pub value: String,
    pub checked: bool,
    pub disabled: bool,
}

#[derive(Debug)]
pub struct Node {
    pub parent: Option<NodeId>,
    pub children: Vec<NodeId>,
    pub node_type: NodeType,
}

#[derive(Debug, Default)]
pub struct Document {
    pub nodes: Vec<Node>,
    pub root: NodeId,
    pub id_index: std::collections::HashMap<String, NodeId>,
}
```

### 19.2 イベントリスナー保持構造

```rust
pub type ListenerId = u64;

pub struct ListenerEntry {
    pub id: ListenerId,
    pub event_type: String,
    pub use_capture: bool,
    pub callback: JsCallbackRef,
}

pub struct ListenerStore {
    // node_id -> listeners
    pub map: std::collections::HashMap<NodeId, Vec<ListenerEntry>>,
    pub next_id: ListenerId,
}
```

要点:
- `removeEventListener` は `event_type + callback + use_capture` で一致削除
- dispatch中にリスナー配列が変更されても安全になるよう、実行対象はスナップショットを使う

### 19.3 ランタイム集約構造

```rust
pub struct Runtime {
    pub document: Document,
    pub listeners: ListenerStore,
    pub js: JsEngine,
    pub task_queue: TaskQueue,
    pub trace: bool,
}
```

`Harness` は `Runtime` をラップし、操作APIとアサートAPIを提供する。

## 20. HTMLロード仕様

1. 受け取ったHTML文字列をparse
2. `document` ノード作成
3. Element/Textを順にArenaへ格納
4. `id`属性を見つけた時点で `id_index` 登録（重複は後勝ちではなくエラー推奨）
5. `<script>`要素のテキストを文書順で収集
6. DOM構築完了後にscriptを同期実行

補足:
- script実行中のDOM変更（`appendChild`等）を許可する場合は、DOM APIの整合性を優先し、`id_index`を都度更新する
- まずは `appendChild/removeChild` を非対応にしてもよい（MVP安定化優先）

## 21. JSブリッジ詳細

### 21.1 オブジェクトモデル
- `DocumentRef` と `ElementRef` をJSへ露出
- 各JSオブジェクトの内部スロットに `NodeId` を保存
- API呼び出し時に `NodeId` の生存確認を行う（削除済みノード対策）

### 21.2 代表APIのRust側シグネチャ

```rust
fn js_document_get_element_by_id(ctx: &mut JsContext, id: String) -> JsValue;
fn js_document_query_selector(ctx: &mut JsContext, selector: String) -> JsValue;
fn js_element_add_event_listener(
    ctx: &mut JsContext,
    this_node: NodeId,
    event_type: String,
    callback: JsValue,
    use_capture: bool,
) -> JsResult<()>;
```

### 21.3 例外方針
- Rust側エラーはJS例外に変換（`TypeError`または`Error`）
- JS例外は `JsException { message, stack }` としてHarnessへ返却

## 22. イベント仕様の厳密化

### 22.1 `click(selector)` の動作順
1. 対象Element解決
2. `disabled=true` なら何もしない（ブラウザ挙動に合わせる）
3. `click` をdispatch
4. `defaultPrevented` が `false` の場合に既定動作
5. 既定動作により必要なら `input`/`change`/`submit` を追加dispatch
6. task queue flush（auto flush設定時）

### 22.2 `type_text(selector, text)` の動作順
1. 対象が `input`/`textarea` であることを検証
2. `value` を `text` に置換
3. `input` dispatch（`bubbles=true`）
4. `change` は呼ばない（`change`は明示イベントやblur相当時）

### 22.3 `set_checked(selector, checked)` の動作順
1. 対象がcheckbox/radioであることを検証
2. 値が変わる場合のみ更新
3. `input` dispatch
4. `change` dispatch

## 23. セレクタエンジン詳細

MVP実装案:
- 文字列を簡易パースして `SelectorAst` を作る
- 右から左へのマッチングで親探索

```rust
enum SimpleSelector {
    Id(String),
    Class(String),
    Tag(String),
    AttrEq { key: String, value: String },
}

struct SelectorStep {
    simple: Vec<SimpleSelector>,
    combinator: Option<Combinator>, // Descendant only in MVP
}
```

性能:
- `#id` は `id_index` 直参照で O(1)
- それ以外は最悪 O(N) 走査

## 24. アサーション失敗フォーマット

```text
AssertionFailed: assert_text
  selector : #result
  expected : "OK:Taro"
  actual   : "NG"
  snippet  : <p id="result">NG</p>
```

設計方針:
- 1回の失敗で原因特定できる情報量を確保
- セレクタ未解決と値不一致は必ず区別

## 25. API契約テスト

最低限追加するべき契約テスト:
1. `querySelector("#id")` が先頭一致要素を返す
2. `addEventListener(capture=true)` がcapture順で呼ばれる
3. `stopPropagation` が親への伝播を止める
4. checkbox `click` で `checked` が反転する
5. `preventDefault` でsubmit既定動作が抑止される
6. `disabled` 要素の `click` が無視される

## 26. 実装順（タスク分解）

1. `dom_core`: Arena/Node/selector/id_index
2. `event_system`: listener登録とdispatch
3. `js_runtime`: `document`, `Element`, listener bridge
4. `runtime_core`: parse -> build -> script実行
5. `test_harness`: action/assert API
6. 仕様テスト整備
7. エラー文言とtrace改善

## 27. 将来拡張ポイント

- `innerHTML` 対応
- `classList` API
- `appendChild/removeChild`
- `setTimeout` + task queue
- `radio` グループ排他
- `form.elements` / `FormData` 的な最小API

拡張時も「必要なユースケース起点でAPIを足す」方針を維持する。

## 28. 提案する最初のマイルストーン

2週間想定:
- Day 1-2: DOM + selector MVP
- Day 3-4: イベントdispatch MVP
- Day 5-6: JS bridge (`getElementById`, `addEventListener`)
- Day 7-8: Harness action/assert
- Day 9-10: 契約テスト + 失敗表示改善

完了条件:
- サンプル3シナリオが `cargo test` で安定通過
- 主要APIでpanicなし、すべて `Result` 返却

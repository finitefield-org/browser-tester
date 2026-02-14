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
- 実ネットワークI/O（XHR/WebSocket/外部HTTP）。`fetch` はモック注入でのみ対応
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
  R --> S["script_runtime (self-implemented)"]
  D <--> S
  E <--> S
```

モジュール:
- `dom_core`: DOM木、セレクタ、属性/プロパティ
- `script_runtime`: 自前パーサ + 自前評価器（JSサブセット）
- `event_system`: イベント伝播と既定動作
- `runtime_core`: 初期化、スクリプト実行、タスクキュー
- `test_harness`: 高水準のテスト操作API

## 5. クレート構成（採用方針）

- 本プロジェクトは**単一クレートで実装する**
- `src/lib.rs` を中心に、必要に応じて同一クレート内でモジュール分割する
- `runtime-core` / `dom-core` などの別クレート分割は行わない

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
- `value: String`（input/textarea/select）
- `checked: bool`（checkbox/radio）
- `disabled: bool`
- `readonly: bool`
- `required: bool`

### 6.2 インデックス
- `id_index: HashMap<String, Vec<NodeId>>`
- `class_index: HashMap<String, Vec<NodeId>>`（必要時）
- `#id` / `getElementById` は同一idの先頭要素を返し、重複idは内部で保持する

### 6.3 セレクタ
MVP対応:
- 単純/複合: `#id`, `.class`, `tag`, `[name]`, `[name=value]`,
  `tag#id.class[attr=value][attr2]`
- 結合子: 子孫（空白）, 子（`>`）, 隣接兄弟（`+`）, 後続兄弟（`~`）
- グループ: `A, B`（重複は除外し、文書順で返す）

非対応セレクタは明示的エラーにする（サイレント無視しない）。

## 7. スクリプトランタイム詳細

### 7.1 実装方式
- 外部JSエンジンは使わない（純Rustの自前実装）
- `<script>`文字列を自前パーサでASTへ変換し、自前評価器で実行
- 対応範囲はテスト用途に必要なJSサブセットへ限定し、非対応構文は`ScriptParse`で明示エラー

### 7.2 対応する構文/DOM API（主要）
- リスナー登録/解除: `addEventListener(...)`, `removeEventListener(...)`
- 制御構文: `if/else`, `while`, `do...while`, `for`, `for...in`, `for...of`, `break`, `continue`, `return`
- 主要演算子: 三項演算子, 論理/比較/厳密比較, 算術, bitwise, 代入演算子（`+=`, `&&=`, `??=` など）
- 数値リテラル: 整数/小数/指数/16進/8進/2進、BigIntリテラル
- DOM参照: `getElementById`, `querySelector`, `querySelectorAll`, `querySelectorAll(...).length`,
  `form.elements.length`, `form.elements[index]`,
  `new FormData(form)`, `formData.get(name)`, `formData.has(name)`,
  `formData.getAll(name).length`
- DOM更新: `textContent`, `value`, `checked`, `disabled`, `readonly`, `required`, `className`, `id`, `name`, `classList.*`,
  `setAttribute/getAttribute/hasAttribute/removeAttribute`, `dataset.*`, `style.*`,
  `matches(selector)`, `closest(selector)`（未一致時は `null`）,
  `getComputedStyle(element).getPropertyValue(property)`,
  `createElement/createTextNode`, `append/appendChild/prepend/removeChild/insertBefore/remove()`,
  `before/after/replaceWith`, `insertAdjacentElement/insertAdjacentText/insertAdjacentHTML`, `innerHTML`
- タイマー: `setTimeout(callback, delayMs?)` / `setInterval(callback, delayMs?)`
  （timer ID返却。実時間待ちは行わず、`harness.advance_time(ms)` / `harness.flush()` で実行）,
  `clearTimeout(timerId)` / `clearInterval(timerId)`,
  `requestAnimationFrame` / `cancelAnimationFrame`, `queueMicrotask`
- 時刻: `Date.now()` / `performance.now()`（fake clockの現在値 `now_ms` を返す）
- 乱数: `Math.random()`（決定論PRNGの浮動小数 `0.0 <= x < 1.0` を返す）
- モック前提API: `fetch`, `matchMedia`, `alert`, `confirm`, `prompt`
- イベント: `preventDefault`, `stopPropagation`, `stopImmediatePropagation`
- `offsetWidth`, `offsetHeight`, `offsetTop`, `offsetLeft`, `scrollWidth`, `scrollHeight`, `scrollTop`, `scrollLeft`（最小実装として数値返却）

#### 7.2.1 非対応DOM APIの優先順位
- 第一優先: テストに必須なDOM参照・更新（`getElementById`, `querySelector*`, `textContent`, `value`, `checked`, `disabled`, `readonly`, `required`, `classList`, `dataset`, `style`, `append*`/`remove*`系）
- 第二優先: タイマー/イベント/フォーム関連（`setTimeout`, `setInterval`, `clearTimeout`, `clearInterval`, `preventDefault`, `FormData`, `submit`）
- 第三優先: `focus` などの表示・計測系API
- 非対応は `ScriptParse`/`ScriptRuntime` レイヤで明示エラーとして失敗させる（静かな無視はしない）
- 優先拡張順は `dataset/style` → DOMイベント周り → `offset/scroll`（読取最小実装） → その他表示・計測系

#### 7.2.2 パーサ判定順（実装メモ）
- `event.currentTarget` と `document.getElementById(...).matches(...)`/`closest(...)` のような式は、`DomRef` 判定以前に
  `event`/`DOMメソッド` 判定を行う（`document.getElementById(...).textContent` の誤解釈を防止）
- この順序で、既知の `ScriptParse` 例外系（`event` と `DOM` の同名プロパティ衝突）を回避している

`FormData` の簡易仕様（テスト用途）:
- `new FormData(form)` は `form.elements` を走査してスナップショットを作る
- 対象は `name` を持つ有効なコントロールのみ（`disabled` と `button/submit/reset/file/image` は除外）
- checkbox/radio は `checked=true` のものだけ対象、`value` が空なら `"on"` を使う
- `.get(name)` は最初の値を返し、存在しない場合は空文字
- `.has(name)` はキー存在判定を返す
- `.getAll(name).length` は同一キーの件数を返す
- `formData.append(name, value)` は末尾に値を追加する（`FormData` 変数に対するstatementのみ対応）
- `textarea` の初期値は要素本文テキストを使う
- `select` の初期値は `selected` 付き `option` を優先し、なければ先頭 `option` を使う
- `option` に `value` 属性がない場合、`option` のテキストを値として使う
- `select.value = x` 代入時は一致する `option` を1つ選択状態にし、他は非選択にする

### 7.3 Rust<->Scriptブリッジ
- ASTノード内の`DomQuery`/`DomProp`を介してDOMへアクセス
- イベント実行時は `EventState` とローカル変数環境 `env` を評価器へ渡す
- DOM更新時は必要に応じて`id_index`を同期する

## 8. イベントシステム詳細

### 8.1 Eventオブジェクト
フィールド:
- `type`, `target`, `currentTarget`, `bubbles`, `cancelable`, `defaultPrevented`, `isTrusted`
- `eventPhase`, `timeStamp`
- 参照用プロパティ: `targetName`, `currentTargetName`, `targetId`, `currentTargetId`
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
2. 画面遷移などの既定動作は行わない（`preventDefault` 状態は `event.defaultPrevented` として観測可能）

## 9. Runtime実行モデル

### 9.1 初期化
1. HTML parse（自前HTMLパーサ）
2. DOM構築
3. `<script>`を文書順で同期実行
4. `<script>` 実行で発生した microtask は各トップレベルタスク終了時に実行（timerは残す）

### 9.2 タスクキュー
- 同期実行を基本としつつ、microtaskキュー（`queueMicrotask` / Promise reaction）を実装
- タイマーは実時間を待たず、fake clock（初期値 `0ms`）で決定論的に実行する
- `harness.advance_time(ms)` で fake clock を進め、`due_at <= now` のタイマーのみ実行する
- `harness.run_due_timers()` で `now_ms` を進めずに、`due_at <= now_ms` のタイマーのみ実行する
- `harness.advance_time_to(targetMs)` で fake clock を絶対時刻へ進め、`due_at <= targetMs` のタイマーを実行する
- `harness.flush()` は fake clock を必要分だけ先送りして、キューが空になるまで実行する
- `harness.run_next_timer()` は次の1件だけ実行し、実行した場合 `true` を返す（空キューは `false`）
- `harness.run_next_due_timer()` は `due_at <= now_ms` の次の1件だけ実行し、実行した場合 `true` を返す
- `harness.clear_timer(timerId)` は指定timer IDを削除し、削除対象が存在した場合 `true` を返す
- `harness.clear_all_timers()` はキュー中タイマーを全削除し、削除件数を返す
- 安全上限は既定で `10000`（`harness.set_timer_step_limit(max_steps)` で変更可能）
- `harness.flush()` / `advance_time()` で安全上限超過時は、
  `now_ms`, `due_limit`, `pending_tasks`, `next_task` を含む診断付きエラーを返す
  （`due_limit` は `flush()` では `none`、`advance_time(ms)` では更新後の `now_ms`）
- `harness.pending_timers()` で現在キュー中のタイマー（`due_at`,`order` 昇順）を取得できる

### 9.3 決定論サポート
- `Date.now()` / `performance.now()` は fake clock（`now_ms`）を返す
- `now_ms` は `advance_time(ms)` / `advance_time_to(ms)` / `flush()` / `run_next_timer()` により進む
- `Math.random()` は決定論PRNGで生成される
- `Harness::set_random_seed(seed)` で乱数列を再現可能にする

## 10. テストハーネスAPI詳細

```rust
pub struct Harness { /* runtime */ }

impl Harness {
    pub fn from_html(html: &str) -> Result<Self>;

    // Action
    pub fn type_text(&mut self, selector: &str, text: &str) -> Result<()>;
    pub fn set_checked(&mut self, selector: &str, checked: bool) -> Result<()>;
    pub fn click(&mut self, selector: &str) -> Result<()>;
    pub fn focus(&mut self, selector: &str) -> Result<()>;
    pub fn blur(&mut self, selector: &str) -> Result<()>;
    pub fn submit(&mut self, selector: &str) -> Result<()>;
    pub fn dispatch(&mut self, selector: &str, event: &str) -> Result<()>;
    pub fn dump_dom(&self, selector: &str) -> Result<String>;

    // Trace
    pub fn enable_trace(&mut self, enabled: bool);
    pub fn take_trace_logs(&mut self) -> Vec<String>;
    pub fn set_trace_stderr(&mut self, enabled: bool);
    pub fn set_trace_events(&mut self, enabled: bool);
    pub fn set_trace_timers(&mut self, enabled: bool);
    pub fn set_trace_log_limit(&mut self, max_entries: usize) -> Result<()>;

    // Determinism / clocks
    pub fn set_random_seed(&mut self, seed: u64);
    pub fn set_timer_step_limit(&mut self, max_steps: usize) -> Result<()>;
    pub fn now_ms(&self) -> i64;
    pub fn advance_time(&mut self, ms: i64) -> Result<()>;
    pub fn advance_time_to(&mut self, target_ms: i64) -> Result<()>;
    pub fn flush(&mut self) -> Result<()>;
    pub fn clear_timer(&mut self, timer_id: i64) -> bool;
    pub fn clear_all_timers(&mut self) -> usize;
    pub fn pending_timers(&self) -> Vec<PendingTimer>;
    pub fn run_due_timers(&mut self) -> Result<usize>;
    pub fn run_next_timer(&mut self) -> Result<bool>;
    pub fn run_next_due_timer(&mut self) -> Result<bool>;

    // Mock / browser-like globals
    pub fn set_fetch_mock(&mut self, url: &str, body: &str);
    pub fn clear_fetch_mocks(&mut self);
    pub fn take_fetch_calls(&mut self) -> Vec<String>;
    pub fn set_match_media_mock(&mut self, query: &str, matches: bool);
    pub fn clear_match_media_mocks(&mut self);
    pub fn set_default_match_media_matches(&mut self, matches: bool);
    pub fn take_match_media_calls(&mut self) -> Vec<String>;
    pub fn enqueue_confirm_response(&mut self, accepted: bool);
    pub fn set_default_confirm_response(&mut self, accepted: bool);
    pub fn enqueue_prompt_response(&mut self, value: Option<&str>);
    pub fn set_default_prompt_response(&mut self, value: Option<&str>);
    pub fn take_alert_messages(&mut self) -> Vec<String>;

    // Assert
    pub fn assert_text(&self, selector: &str, expected: &str) -> Result<()>;
    pub fn assert_value(&self, selector: &str, expected: &str) -> Result<()>;
    pub fn assert_checked(&self, selector: &str, expected: bool) -> Result<()>;
    pub fn assert_exists(&self, selector: &str) -> Result<()>;
}
```

```rust
pub struct PendingTimer {
    pub id: i64,
    pub due_at: i64,
    pub order: i64,
    pub interval_ms: Option<i64>,
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
- `HtmlParse { message }`
- `ScriptParse { message }`
- `ScriptRuntime { message }`
- `SelectorNotFound { selector }`
- `UnsupportedSelector { selector }`
- `TypeMismatch { selector, expected, actual }`
- `AssertionFailed { selector, expected, actual, dom_snippet }`

失敗時は次を必ず含める:
- 対象セレクタ
- 期待値/実値
- 対象ノード周辺のHTML断片（最大200文字）

## 12. ログ・デバッグ

- `Harness::enable_trace(true)` でイベントトレース有効化
- トレースは標準エラーへ出力され、`take_trace_logs()` で取得してクリアできる
- `set_trace_stderr(false)` で標準エラー出力を止め、ログ収集のみを有効化できる
- `set_trace_events(false)` / `set_trace_timers(false)` でカテゴリ単位のログ出力を制御できる
- 保持件数は既定 `10000`。`set_trace_log_limit(n)` で変更でき、超過時は古いログから捨てる
- タイマー制御API実行時はサマリ行（advance/advance_to/run_due/flush）を出力する
- 出力例:
  - `[event] click target=#submit current=#submit phase=bubble default_prevented=false`
  - `[event] done submit target=#signup current=#signup outcome=completed default_prevented=false propagation_stopped=false immediate_stopped=false`
  - `[timer] schedule timeout id=1 due_at=10 delay_ms=10`
  - `[timer] run id=1 due_at=10 interval_ms=none now_ms=10`
  - `[timer] advance delta_ms=5 from=0 to=5 ran_due=1`
  - `[timer] flush from=5 to=10 ran=1`

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

実装方針:
- HTML parse: 自前実装
- Selector: 自前実装
- Script runtime: 自前パーサ + 自前評価器
- Error: 独自 `Error` enum
- 外部依存は最小限（`regex`, `num-bigint`, `num-traits`）

## 17. 既知リスクと対策

1. JS互換性不足（ES機能差）
- 対策: 対象HTMLのJS制約を定義し、非対応構文を早期エラー化

2. DOM仕様の実装漏れ
- 対策: 必須Web API一覧を契約化して段階実装

3. イベント順序のズレ
- 対策: 仕様テストを先に固定し、変更時にCIで検出

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
    pub readonly: bool,
    pub required: bool,
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
    pub id_index: std::collections::HashMap<String, Vec<NodeId>>,
}
```

### 19.2 イベントリスナー保持構造

```rust
pub struct ListenerEntry {
    pub capture: bool,
    pub callback: ScriptHandler,
}

pub struct ListenerStore {
    // node_id -> event_type -> listeners
    pub map: std::collections::HashMap<
        NodeId,
        std::collections::HashMap<String, Vec<ListenerEntry>>,
    >,
}
```

要点:
- `removeEventListener` は `event_type + callback + capture` で一致削除
- dispatch中にリスナー配列が変更されても安全になるよう、実行対象はスナップショットを使う

### 19.3 ランタイム集約構造

```rust
pub struct Runtime {
    pub dom: Dom,
    pub listeners: ListenerStore,
    pub script_env: std::collections::HashMap<String, Value>,
    pub task_queue: Vec<ScheduledTask>,
    pub microtask_queue: std::collections::VecDeque<ScheduledMicrotask>,
    pub trace: bool,
    pub trace_events: bool,
    pub trace_timers: bool,
    pub trace_logs: Vec<String>,
    pub trace_log_limit: usize,
    pub trace_to_stderr: bool,
}
```

`Harness` は `Runtime` をラップし、操作APIとアサートAPIを提供する。

## 20. HTMLロード仕様

1. 受け取ったHTML文字列をparse
2. `document` ノード作成
3. Element/Textを順にArenaへ格納
4. `id`属性を見つけた時点で `id_index` 登録（重複idは `Vec<NodeId>` として保持）
5. `<script>`要素のテキストを文書順で収集
6. DOM構築完了後にscriptを同期実行

補足:
- script実行中のDOM変更（`appendChild/removeChild/insertBefore`等）は、DOM APIの整合性を優先し、
  `id_index`を都度更新する

## 21. スクリプト実行詳細

### 21.1 実行モデル
- `<script>`を文単位で解析して `Stmt` / `Expr` のASTへ変換
- リスナー本文は `Stmt` / `Expr` のASTへ変換して保持
- イベント発火時に `execute_stmts` でASTを評価し、DOMへ副作用を反映

### 21.2 代表APIのRust側シグネチャ

```rust
fn parse_block_statements(body: &str) -> Result<Vec<Stmt>>;
fn parse_single_statement(stmt: &str) -> Result<Stmt>;
fn execute_stmts(
    &mut self,
    stmts: &[Stmt],
    event_param: &Option<String>,
    event: &mut EventState,
    env: &mut std::collections::HashMap<String, Value>,
) -> Result<()>;
```

### 21.3 例外方針
- 構文エラーは `ScriptParse`
- 実行時エラーは `ScriptRuntime`
- 失敗時はセレクタ・期待値/実値を返す（Assertion系）

## 22. イベント仕様の厳密化

### 22.1 `click(selector)` の動作順
1. 対象Element解決
2. `disabled=true` なら何もしない（ブラウザ挙動に合わせる）
3. `click` をdispatch
4. `defaultPrevented` が `false` の場合に既定動作
5. 既定動作により必要なら `input`/`change`/`submit` を追加dispatch
6. トップレベルタスク終了時に microtask queue を自動実行

### 22.2 `type_text(selector, text)` の動作順
1. 対象が `input`/`textarea` であることを検証
2. `disabled` / `readonly` の場合は何もしない
3. `value` を `text` に置換
4. `input` dispatch（`bubbles=true`）
5. `change` は呼ばない（`change`は明示イベントやblur相当時）

### 22.3 `set_checked(selector, checked)` の動作順
1. 対象がcheckbox/radioであることを検証
2. 値が変わる場合のみ更新
3. `input` dispatch
4. `change` dispatch

## 23. セレクタエンジン詳細

MVP実装案:
- 文字列を簡易パースして `SelectorAst` を作る
- 右から左へのマッチングで親探索
- 対応セレクタ: `#id`, `.class`, `タグ`, `[attr]`, `[attr='value']`, `*`,
  `:first-child`, `:last-child`, `:first-of-type`, `:last-of-type`,
  `:only-child`, `:only-of-type`,
  `:nth-child(n)`, `:nth-child(odd)`, `:nth-child(even)`, `:nth-child(an+b)`,
  `:nth-last-child(n|odd|even|an+b)`,
  `:nth-of-type(n|odd|even|an+b)`, `:nth-last-of-type(n|odd|even|an+b)`,
  `:empty`,
  `:checked`, `:disabled`, `:enabled`, `:required`, `:optional`,
  `:read-only`（非標準別名 `:readonly` 対応）,
  `:read-write`, `:focus`, `:focus-within`, `:active`,
  `:not(selector)`, `:is(selector)`, `:where(selector)`, `:has(selector)`（selector-list 対応）など,
  子孫/子/隣接/一般兄弟結合子
- `:nth-child(an+b)` は `2n+1`, `-n+3`, `n+1` などをサポート。`n` は1ベース要素インデックス。
- `:nth-last-child(an+b|odd|even|n)` も同様に1ベース要素インデックスの末尾基準でサポート。
- 属性演算子は `=`, `^=`, `$=`, `*=`, `~=`, `|=` をサポート

```rust
enum SelectorPseudoClass {
    FirstChild,
    LastChild,
    FirstOfType,
    LastOfType,
    OnlyChild,
    OnlyOfType,
    Checked,
    Disabled,
    Enabled,
    Required,
    Optional,
    Readonly,
    Readwrite,
    Empty,
    Focus,
    FocusWithin,
    Active,
    Is(Vec<Vec<SelectorPart>>),
    Where(Vec<Vec<SelectorPart>>),
    Has(Vec<Vec<SelectorPart>>),
    NthOfType(NthChildSelector),
    NthLastOfType(NthChildSelector),
    Not(Vec<Vec<SelectorPart>>),
    NthChild(NthChildSelector),
    NthLastChild(NthChildSelector),
}

enum NthChildSelector {
    Exact(usize),
    Odd,
    Even,
    AnPlusB(i64, i64),
}

struct SelectorStep {
    tag: Option<String>,
    universal: bool,
    id: Option<String>,
    classes: Vec<String>,
    attrs: Vec<SelectorAttrCondition>,
    pseudo_classes: Vec<SelectorPseudoClass>,
}

enum SelectorCombinator {
    Descendant,
    Child,
    AdjacentSibling,
    GeneralSibling,
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

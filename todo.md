# browser-tester 未実装タスク一覧

この一覧は実装上の未完了項目およびテストカバレッジ不足を示します。

## 1. ブラウザ互換グローバル関数の追加 (実装 / テスト)

- [x] `encodeURI` をサポートする
- [x] `encodeURIComponent` をサポートする
- [x] `decodeURI` をサポートする
- [x] `decodeURIComponent` をサポートする
- [x] `escape` をサポートする
- [x] `unescape` をサポートする
- [x] `isFinite` をサポートする
- [x] `isNaN` をサポートする
- [x] `parseInt` をサポートする
- [x] `parseFloat` をサポートする
- [x] `atob` をサポートする
- [x] `btoa` をサポートする
- [x] `window`/`encodeURI` などブラウザ関数のエラーメッセージと仕様一致を確認するテストを追加する

## 2. 追加のブラウザグローバル関数 (実装 / テスト)

- [ ] `requestAnimationFrame` / `cancelAnimationFrame` をサポートする
- [ ] `fetch` をサポートする（最低限 `Response.text()` / `Response.json()`）
- [ ] `structuredClone` をサポートする
- [ ] `matchMedia` をサポートする
- [ ] `alert` / `confirm` / `prompt` をサポートする（テスト用モックを含む）
- [ ] `eval` をサポートする（許容スコープを明確化する）
- [ ] `performance.now` をサポートする
- [ ] `JSON.parse` / `JSON.stringify` をサポートする

## 3. `String` オブジェクト関数 (実装 / テスト)

- [ ] `trim` / `trimStart` / `trimEnd` をサポートする
- [ ] `toUpperCase` / `toLowerCase` をサポートする
- [ ] `includes` / `startsWith` / `endsWith` をサポートする
- [ ] `slice` / `substring` をサポートする
- [ ] `split` / `replace` をサポートする
- [ ] `indexOf` をサポートする

## 4. `Date` オブジェクト関数 (実装 / テスト)

- [ ] `new Date()` / `new Date(value)` をサポートする
- [ ] `Date.parse` / `Date.UTC` をサポートする
- [ ] `getTime` / `setTime` をサポートする
- [ ] `toISOString` をサポートする
- [ ] `getFullYear` / `getMonth` / `getDate` をサポートする
- [ ] `getHours` / `getMinutes` / `getSeconds` をサポートする

## 5. `Array` オブジェクト関数 (実装 / テスト)

- [ ] 配列リテラル (`[]`) と添字アクセスをサポートする
- [ ] `Array.isArray` をサポートする
- [ ] `length` / `push` / `pop` / `shift` / `unshift` をサポートする
- [ ] `map` / `filter` / `reduce` をサポートする
- [ ] `forEach` / `find` / `some` / `every` / `includes` をサポートする
- [ ] `slice` / `splice` / `join` をサポートする

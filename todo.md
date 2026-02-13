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
- [x] `fetch` をサポートする（テスト用途向けにモックへ差し替え可能な設計）
- [ ] `structuredClone` をサポートする
- [ ] `matchMedia` をサポートする
- [x] `alert` / `confirm` / `prompt` をサポートする（`confirm`/`prompt` の返り値モックを含む）
- [x] `JSON.parse` / `JSON.stringify` をサポートする

## 3. `String` オブジェクト関数 (実装 / テスト)

- [x] `trim` / `trimStart` / `trimEnd` をサポートする
- [x] `toUpperCase` / `toLowerCase` をサポートする
- [x] `includes` / `startsWith` / `endsWith` をサポートする
- [x] `slice` / `substring` をサポートする
- [x] `split` / `replace` をサポートする
- [x] `indexOf` をサポートする

## 4. `Date` オブジェクト関数 (実装 / テスト)

- [x] `new Date()` / `new Date(value)` をサポートする
- [x] `Date.parse` / `Date.UTC` をサポートする
- [x] `getTime` / `setTime` をサポートする
- [x] `toISOString` をサポートする
- [x] `getFullYear` / `getMonth` / `getDate` をサポートする
- [x] `getHours` / `getMinutes` / `getSeconds` をサポートする

## 5. `Array` オブジェクト関数 (実装 / テスト)

- [x] 配列リテラル (`[]`) と添字アクセスをサポートする
- [x] `Array.isArray` をサポートする
- [x] `length` / `push` / `pop` / `shift` / `unshift` をサポートする
- [x] `map` / `filter` / `reduce` をサポートする
- [x] `forEach` / `find` / `some` / `every` / `includes` をサポートする
- [x] `slice` / `splice` / `join` をサポートする

## 6. `Object` オブジェクト関数 (実装 / テスト)

- [x] オブジェクトリテラル (`{}`) をサポートする
- [x] プロパティ参照 (`obj.key`) と代入 (`obj.key = ...`, `obj['key'] = ...`) をサポートする
- [x] `Object.keys` / `Object.values` / `Object.entries` をサポートする
- [x] `Object.hasOwn` / `obj.hasOwnProperty` をサポートする

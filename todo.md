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

- [x] `requestAnimationFrame` / `cancelAnimationFrame` をサポートする
- [x] `fetch` をサポートする（テスト用途向けにモックへ差し替え可能な設計）
- [x] `structuredClone` をサポートする
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

## 7. `Math` 定数・静的メソッド (実装 / テスト)

- [x] `Math.E` / `LN10` / `LN2` / `LOG10E` / `LOG2E` / `PI` / `SQRT1_2` / `SQRT2` をサポートする
- [x] `Math[Symbol.toStringTag]` をサポートする
- [x] 主要静的メソッド（`abs` / `acos` / `acosh` / `asin` / `asinh` / `atan` / `atan2` / `atanh` / `cbrt` / `ceil` / `clz32` / `cos` / `cosh` / `exp` / `expm1` / `floor` / `f16round` / `fround` / `hypot` / `imul` / `log` / `log10` / `log1p` / `log2` / `max` / `min` / `pow` / `random` / `round` / `sign` / `sin` / `sinh` / `sqrt` / `sumPrecise` / `tan` / `tanh` / `trunc`）をサポートする

## 8. JavaScript演算子の未実装項目 (実装 / テスト)

- [ ] `await` 演算子をサポートする
- [ ] `??` (nullish coalescing) をサポートする
- [ ] `&&=` / `||=` / `??=` をサポートする
- [ ] 分割代入 (`[a, b] = arr`, `{ a, b } = obj`) をサポートする
- [ ] `yield` / `yield*` をサポートする
- [ ] spread syntax (`...obj`) をサポートする
- [ ] comma operator (`,`) をサポートする

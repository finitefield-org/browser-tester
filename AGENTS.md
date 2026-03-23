必要に応じ、テスト用途にAPIをモックできる作りにしてください。
その場合、README.md と doc/mock-guide.md にそのモックの使い方を記載してください。
HTMLの仕様は `html-standard` フォルダにあるので適宜参照してください。

新しい test-only mock を追加するときは、必ず次をセットで行ってください。

- 公開 API の追加または更新
- 最小使用例の追加
- 失敗系を含むテストの追加
- call capture または artifact capture の説明追加
- README.md の更新
- doc/mock-guide.md の更新

新しい公開 API、特に `Harness` のメソッドを追加または変更するときは、必ず次を確認してください。

- それは本当に公開 API か
- 既存 API の組み合わせで足りないか
- test-only mock に分類すべきではないか
- debug / trace 用 API として分けるべきではないか
- `doc/capability-matrix.md` を更新したか
- `README.md` を更新したか
- public contract test を追加または更新したか
- regression test を追加または更新したか

公開 API の置き場は、実装前に `doc/subsystem-map.md` を見て決めてください。


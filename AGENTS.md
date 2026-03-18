必要に応じ、テスト用途にAPIをモックできる作りにしてください。
その場合、README.md と doc/mock-guide.md にそのモックの使い方を記載してください。

新しい test-only mock を追加するときは、必ず次をセットで行ってください。

- 公開 API の追加または更新
- 最小使用例の追加
- 失敗系を含むテストの追加
- call capture または artifact capture の説明追加
- README.md の更新
- doc/mock-guide.md の更新

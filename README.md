# ai-yagni-data-sdk-example-extension-rust

[ai-yagni-data-sdk-rust](https://github.com/pad01g/ai-yagni-data-sdk-rust) の層2の例です。
SDK の基礎語彙（層1）を、EC の業務の形で拡張したフレームワークになっています。

- **構成子**: 表と列の名前を受け取り、業務の言葉（語）を作ります。
  - 構成子は [`Catalog`](ec/src/catalog.rs)・[`Orders`](ec/src/orders.rs)・[`Customers`](ec/src/customers.rs)・[`Payments`](ec/src/payments.rs) です。
  - どの語も、一段たどれば層1の言明（スキーマの事実か、違反を探す SQL ファイル）に落ちます。
- **SQL**: SQL は [`ec/src/sql/`](ec/src/sql) に置き、表と列は `{{名前}}` で差し込みます。このフレームワークのソースは SQL を文字列で持ちません（テストで確かめています）。
- **当てはめ**: 構成子をプロジェクトの表名で呼び、語を並べる作業は、利用者のプロジェクトがします。
  - 例: [ai-yagni-data-sdk-example-backend-rust](https://github.com/pad01g/ai-yagni-data-sdk-example-backend-rust) の `datafutures/src/shop.rs`

## 語

| 構成子 | 語 | 意味 |
|---|---|---|
| Catalog | `prices_are_whole_minor_units` | 価格は通貨の最小単位の整数 |
| | `stock_floor_is_named` | 在庫の下限は名前の付いた CHECK 制約 |
| | `prices_carry_currency(col)` | 価格が通貨を持つ |
| | `stock_allows_backorder(col)` | 在庫を超えて受けられる予約の上限がある |
| | `sold_as_variants(&Variants)` | どの商品も、少なくとも 1 つのバリエーションとして売る |
| Orders | `order_amounts_are_whole_minor_units` | 合計と単価は最小単位の整数 |
| | `lines_copy_the_price` | 明細は注文した時点の単価の写しを持つ |
| | `order_totals_match_lines` | 合計は明細の数量 × 単価の和 |
| | `orders_are_keyed_by_request` | 顧客ごとの依頼の冪等キーで一意 |
| | `lines_refer_to_products` | 明細は商品を ID で指す |
| | `lines_have_their_own_key` | 明細は行番号で区別する（同じ商品を何行でも入れられる） |
| | `order_status_is_named` | 状態の候補は名前の付いた CHECK 制約 |
| | `orders_are_kept_as_records` | 注文と明細は消せず、明細は書き換えられない |
| | `orders_carry_currency(col)` | 注文が通貨を持つ |
| | `order_taxes_match_rates(tax, rate)` | 税は明細ごとの切り捨ての和（`tax` と同じ規則） |
| | `lines_name_variants(col, &Variants)` | 明細はバリエーションを指せる |
| | `lines_track_shipment(col)` | 明細ごとに発送した数を持つ |
| | `shipped_orders_are_fully_shipped(col)` | 発送済みの注文は全数を発送している |
| Customers | `pii_stays_on_customers` | 個人情報は顧客の表にだけある |
| | `orders_refer_to_customers` | 注文は顧客を ID で指す |
| | `erasing_a_customer_takes_their_orders` | 顧客を消すと注文と明細も消える |
| | `orders_outlive_customers` | 顧客を消しても注文は道連れにならない |
| Payments | `orders_carry_a_payment` | 注文は決済の参照を持つ |
| | `payments_are_used_once` | 1 つの決済は 1 つの注文にだけ使う |

## Verus

[`verified/`](verified/src/lib.rs) は、税の端数処理 `tax`（数量 × 単価 × 税率 ÷ 10000 の切り捨て）を証明しています。

- 積があふれないときは仕様どおりの値を返す。
- 税率が 100% 以下なら、税は税抜きの額を超えない。

語 `order_taxes_match_rates` は、DB の上で同じ規則を SQL（[`tax_mismatch.sql`](ec/src/sql/tax_mismatch.sql)）で確かめます。テストでは、両者が同じ値になることも確かめています。

## 動かし方（ホストには docker 以外何も入れない）

```sh
make test     # テスト（EC の形のスキーマを本物の MySQL に当て、語が成り立つ / 壊れた形で成り立たないことを確かめる）
make lint     # rustfmt と clippy
make verify   # Verus（amd64 のイメージ）
make ci       # 全部
```

## 使い方（Cargo）

```toml
[dependencies]
ai-yagni-data-sdk = { git = "https://github.com/pad01g/ai-yagni-data-sdk-rust", tag = "v0.1.0" }
ai-yagni-data-ec = { git = "https://github.com/pad01g/ai-yagni-data-sdk-example-extension-rust", tag = "v0.1.0" }
```

## ライセンス

MIT

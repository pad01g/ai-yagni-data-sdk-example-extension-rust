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
| | `stock_floor_is_named` | 在庫の下限は名前の付いた CHECK 制約にあり、在庫が 0 を下回る商品はない |
| | `prices_carry_currency(col)` | 価格が通貨を持つ |
| | `stock_allows_backorder(col)` | 在庫を超えて受けられる予約の上限がある |
| | `sold_as_variants(&Variants)` | どの商品も、少なくとも 1 つのバリエーションとして売る |
| Orders | `order_amounts_are_whole_minor_units` | 合計と、明細が写した注文時の単価は最小単位の整数 |
| | `order_totals_match_lines` | 合計は明細の数量 × 単価の和 |
| | `orders_are_keyed_by_request` | 顧客ごとの依頼の冪等キーで一意（キーは NULL にならない） |
| | `lines_refer_to_products` | 明細は商品を ID で指す |
| | `lines_have_their_own_key` | 明細は行番号で区別する（同じ商品を何行でも入れられる） |
| | `order_status_is_named` | 状態の候補を守る CHECK 制約に名前が付いている（中身は見ない） |
| | `orders_are_kept_as_records(&Customers)` | 注文と明細は消せず、金額と相手・明細の中身は書き換えられない。顧客を消しても連鎖して消えない |
| | `orders_carry_currency(col)` | 注文が通貨を持つ |
| | `lines_share_the_order_currency(col, &Catalog, col)` | 明細の商品の通貨は注文の通貨と同じ |
| | `order_taxes_match_rates(tax, rate)` | 税率は 0〜100%。税は明細ごとの切り捨ての和（`tax` と同じ規則） |
| | `lines_name_variants(col, &Variants)` | 明細はバリエーションを指せる。指すならその明細の商品のもの |
| | `lines_track_shipment(col)` | 明細ごとに発送した数を持つ |
| | `shipped_orders_are_fully_shipped(col, status)` | 発送済み（状態の値 `status`）の注文は全数を発送している |
| Customers | `pii_stays_on_customers` | 個人情報は顧客の表にだけある |
| | `orders_refer_to_customers` | 注文は顧客を ID で指す |
| | `erasing_a_customer_takes_their_orders` | 顧客を消すと注文と明細も消える |
| | `orders_outlive_customers` | 顧客を消しても注文は道連れにならない |
| Payments | `settled_orders_carry_a_payment` | 決済が通った注文は決済の参照を持つ |
| | `payments_are_used_once` | 1 つの決済は 1 つの注文にだけ使う |

語の確かめ方で、知っておくとよいこと:

- **名前しか見ないもの**: CHECK 制約（`check_exists`）とトリガー（`guarded_against`）は、あることだけを見ます。
  何を拒むかは、未来の rejects（書き込んで拒まれることを確かめる probe）で確かめます。
  - `stock_floor_is_named` は、今の行に負の在庫がないことも SQL で確かめます。
  - `settled_orders_carry_a_payment` は、名前の付いた CHECK 制約が参照の列を `IS NOT NULL` で条件に含むことを、information_schema の条件の文で確かめます。
    参照を持たなくてよい状態（決済の前など）はプロジェクトごとに違うので、状態の一覧は受け取りません。
- **NULL**: 違反を探す SQL は `<=>` で比べ、比べる値が NULL の行も違反として数えます。
- **個人情報の列**: `Customers::pii` は列の名前の一部です（大文字と小文字は区別しない）。`email` は `notify_email`・`email_address`・`billing_email_hash` も見つけます。
  略した名前は、その形も並べます（`addr` は `address` も `ship_to_addr` も含む）。名前に表れない個人情報は見つけられません。
- **状態の値**: `shipped_orders_are_fully_shipped` の状態の値は SQL の文字列として差し込むので、英数字と `_` だけでできていなければなりません。
- **まだない列**: 構造の言明（列や表があるか）の後にデータの言明（その列を読む SQL）を並べる語は、前から順に確かめ、
  成り立たないところで止めます。列がまだなければ、SQL の失敗（Error）でなく、列がないこと（Fail）を理由にします。
- **未来が来たか**: [`all_of`](ec/src/lib.rs) は、語を並べて 1 つの語にします（未来の `realized_if` に、then の語をすべて渡すため）。
  同じく前から順に確かめるので、構造の語を前に並べます。

## テストの EC の形

テスト（[`ec/tests/framework.rs`](ec/tests/framework.rs)）は、[`ec/tests/fixtures/shop/`](ec/tests/fixtures/shop) のスキーマと代表データの上で語を確かめます。
これは [ai-yagni-data-sdk-example-backend-rust](https://github.com/pad01g/ai-yagni-data-sdk-example-backend-rust) の `schema.sql` と `fixtures/0001_shop.sql` のバイト単位の写しです。

- 写しは `ai_yagni_data_ec::shop::{SCHEMA, DATA}` として公開しています。
- backend-rust のテスト（`datafutures` の `the_extension_tests_the_same_shop`）が、自分の正本と比べます。
- backend-rust の正本が変わると、そのテストが失敗します。写しを書き換えて、このリポジトリのタグを上げてください。

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
ai-yagni-data-sdk = { git = "https://github.com/pad01g/ai-yagni-data-sdk-rust", tag = "v0.2.0" }
ai-yagni-data-ec = { git = "https://github.com/pad01g/ai-yagni-data-sdk-example-extension-rust", tag = "v0.2.0" }
```

## ライセンス

MIT

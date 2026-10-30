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
| | `stock_stays_above_its_floor` | 在庫の下限は名前の付いた CHECK 制約にあり、今の行にも在庫が 0 を下回る商品はない（名前とデータの両方を見る） |
| | `prices_carry_currency(col)` | 価格が通貨を持つ |
| | `stock_allows_backorder(col)` | 在庫を超えて受けられる予約の上限がある。在庫の列は商品の表にだけある（上限と同じ表） |
| | `sold_as_variants(&Variants)` | どの商品も、少なくとも 1 つのバリエーションとして売る |
| Orders | `order_amounts_are_whole_minor_units` | 合計と、明細が写した注文時の単価は最小単位の整数 |
| | `order_totals_match_lines` | 合計は明細の数量 × 単価の和 |
| | `orders_are_keyed_by_request` | 顧客ごとの依頼の冪等キーで一意（キーは NULL にならない） |
| | `lines_refer_to_products` | 明細は商品を ID で指す |
| | `lines_have_their_own_key` | 明細は行番号で区別する（`(注文, 商品)` もその一部も一意にしない） |
| | `order_status_is_named` | 状態の候補を守る CHECK 制約に名前が付いている（中身は見ない） |
| | `orders_are_kept_as_records(&Customers, &RecordGuards)` | 注文と明細は消せず、金額と相手・明細の中身は書き換えられず、状態は前にだけ進み、決着した注文に明細は足せない（名前の付いたトリガー）。顧客を消しても連鎖して消えない |
| | `orders_carry_currency(col)` | 注文が通貨を持つ |
| | `lines_share_the_order_currency(col, line_col, guard)` | 明細は注文した時点の通貨を写して持ち、それは注文の通貨と同じ（守りは名前 `guard` の INSERT の前のトリガー） |
| | `order_taxes_match_rates(tax, rate)` | 税率は 0〜100%。税は明細ごとの切り捨ての和（`tax` と同じ規則）。単価と合計は税抜きで、請求する額は合計 + 税 |
| | `lines_name_variants(col, &Variants)` | 明細はバリエーションを指せる。指すならその明細の商品のもの |
| | `lines_track_shipment(col)` | 明細ごとに発送した数を持つ |
| | `shipped_orders_are_fully_shipped(col, status, &ShipmentGuards)` | 発送済み（状態の値 `status`）の注文は全数を発送している（注文の側と明細の側の、名前の付いたトリガー） |
| Customers | `pii_stays_on_customers` | 個人情報は顧客の表にだけある |
| | `orders_refer_to_customers` | 注文は顧客を ID で指す |
| | `erasing_a_customer_takes_their_orders(&Orders, guard)` | 顧客を消すと注文と明細も消える。決着していない注文のある顧客は消せない（名前 `guard` の DELETE の前のトリガー） |
| | `orders_outlive_customers` | 顧客を消しても注文は道連れにならない |
| Payments | `settled_orders_carry_a_payment` | 決済が通った注文は決済の参照を持つ |
| | `payments_are_used_once` | 1 つの決済は 1 つの注文にだけ使う |

`Payments` は `Orders::payments(reference, settled_check)` で作ります（参照を持つ表は注文の表で、表の名前を 2 度書かない）。

語の確かめ方で、知っておくとよいこと:

- **名前しか見ないもの**: CHECK 制約（`check_exists`）とトリガーは、あることだけを見ます。
  何を拒むかは、未来の `rejects_with`（書き込んで、その名前で拒まれることを確かめる probe）と、利用者のプロジェクトの今の probe で確かめます。
  - トリガーは名前で見ます（[`trigger_missing.sql`](ec/src/sql/trigger_missing.sql)。表・イベント・BEFORE も合っていること）。
    SDK の `guarded_against` は名前を見ないので、中身の空なトリガーや、ほかの未来が同じ表に足したトリガー（通貨や発送の守り）でも成り立ってしまうためです。
    守りのトリガーの名前は、`RecordGuards`・`ShipmentGuards` と引数で渡します。
  - トリガーの SIGNAL の文は、自分の名前で始めます（`orders_no_delete: ...`）。`rejects_with` がその名前で、どのトリガーが拒んだかを確かめられます。
  - `stock_stays_above_its_floor` は、今の行に負の在庫がないことも SQL で確かめます（0.2 までの `stock_floor_is_named`。名前だけを見る語でないので名前を変えた）。
  - `settled_orders_carry_a_payment` は、名前の付いた CHECK 制約の条件の文に、参照の列が `` `payment_ref` is not null `` の形（逆引用符で区切った識別子）で現れることを確かめます。
    名前の一部が同じ別の列（`x_payment_ref`）では成り立ちません。条件の意味（どの状態で持つか）は見ません。今の probe で確かめます。
    参照を持たなくてよい状態（決済の前など）はプロジェクトごとに違うので、状態の一覧は受け取りません。
- **明細の写し**: 明細は単価と同じく、注文した時点の通貨も写して持ちます。`lines_share_the_order_currency` は明細の写しを注文の通貨と比べ、商品の今の通貨とは比べません（商品を売り直しても過去の注文は変わらない）。
- **請求する額**: `order_taxes_match_rates` の税は、税抜きの単価から明細ごとに計算します。請求する額は「合計 + 税」です。
  呼ぶ側（決済を頼む側）が何を請求するかは、DB の語では確かめられません。利用者のプロジェクトが、請求する固定クエリやコードを見張ってください。
- **在庫と予約の上限**: `stock_allows_backorder` は、在庫の列（`stock` か `_stock` で終わる名前）が商品の表の外にあれば成り立ちません。在庫をバリエーションへ移す未来とは、そのままでは両立しません。
- **NULL**: 違反を探す SQL は `<=>` で比べ、比べる値が NULL の行も違反として数えます。
- **個人情報の列**: `Customers::pii` は列の名前の一部です（大文字と小文字は区別しない）。`email` は `notify_email`・`email_address`・`billing_email_hash` も見つけます。
  - 略した名前は、その形も並べます（`addr` は `address` も `ship_to_addr` も含む。`tel` は `telephone` も含む）。
  - 短すぎる一部は、個人情報でない列まで拾います（`name` は商品の `name` も拾う）。氏名は `full_name` や `recipient` のように、ほかの表に現れない形で並べます。
  - 名前に表れない個人情報は見つけられません。
- **状態の値**: `shipped_orders_are_fully_shipped` の状態の値は SQL の文字列として差し込むので、英数字と `_` だけでできていなければなりません。
- **まだない列**: 構造の言明（列や表があるか）の後にデータの言明（その列を読む SQL）を並べる語は、SDK の `seq` で前から順に確かめ、
  成り立たないところで止めます。列がまだなければ、SQL の失敗（Error）でなく、列がないこと（Fail）を理由にします。
  `seq` は材料を残すので、材料の書き間違いも検査の規則の節に出ます（0.2 までのこのフレームワークの `in_order` は材料を残さなかった）。
- **未来が来たか**: [`all_of`](ec/src/lib.rs) は、語を並べて 1 つの語にします（未来の `realized_if` に、then の語をすべて渡すため）。
  これも `seq` で、前から順に確かめ、成り立たない語（確かめられない語も）で止めます。止まった後ろの語は確かめないので、構造の語を前に並べます。

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
ai-yagni-data-sdk = { git = "https://github.com/pad01g/ai-yagni-data-sdk-rust", tag = "v0.3.0" }
ai-yagni-data-ec = { git = "https://github.com/pad01g/ai-yagni-data-sdk-example-extension-rust", tag = "v0.3.0" }
```

## 0.2 から 0.3 への変更（互換でない点）

- SDK の v0.3.0 を使います。
- `stock_floor_is_named` は `stock_stays_above_its_floor` になりました。
- `Payments` は構造体の式で作れなくなりました。`Orders::payments(reference, settled_check)` で作ります。
- 引数が増えた語: `orders_are_kept_as_records`（`&RecordGuards`）、`shipped_orders_are_fully_shipped`（`&ShipmentGuards`）、
  `erasing_a_customer_takes_their_orders`（決着していない注文を守るトリガーの名前）。
- `lines_share_the_order_currency` は、`(注文の通貨の列, 明細の通貨の列, 明細の INSERT の前のトリガーの名前)` を受け取ります（`&Catalog` と商品の通貨の列は受け取らない）。
- `lines_have_their_own_key` は `not(unique_within(..))` で見ます（`(注文, 商品)` の一部だけの一意索引でも成り立たない）。
- `settled_orders_carry_a_payment` は、参照の列を識別子の区切りまで見ます。
- `stock_allows_backorder` は、在庫の列が商品の表にだけあることも見ます。
- `all_of` は SDK の `seq` で作ります（材料の語が残る）。

## ライセンス

MIT

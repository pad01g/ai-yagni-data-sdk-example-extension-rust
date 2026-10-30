//! フレームワークの語が、EC の形のスキーマ（tests/fixtures/shop）の上で成り立ち、壊れた形では成り立たないこと。
//! 層2の規則（SQL を文字列で持たない、SDK の層1だけを使う）もここで確かめる。
//!
//! MySQL は `AI_YAGNI_MYSQL` のサーバーを使う（`make test` が `with-mysql cargo test` で立てる）。
use std::path::Path;

use ai_yagni_data_ec::{
    Catalog, Customers, Lines, Orders, Payments, RecordGuards, ShipmentGuards, Variants, all_of, shop, tax,
};
use ai_yagni_data_sdk::lint;
use ai_yagni_data_sdk::mysql::{ENV, Scratch, Server};
use ai_yagni_data_sdk::vocab::{Verdict, Word};

const CATALOG: Catalog =
    Catalog { table: "products", id: "id", price: "price", stock: "stock", stock_floor: "products_stock_nonnegative" };
const ORDERS: Orders = Orders {
    table: "orders",
    id: "id",
    customer: "customer_id",
    request_key: "request_key",
    total: "total",
    status: "status",
    status_check: "orders_status",
    lines: Lines {
        table: "order_items",
        order: "order_id",
        line_no: "line_no",
        product: "product_id",
        quantity: "quantity",
        unit_price: "unit_price",
    },
};
const CUSTOMERS: Customers = Customers {
    table: "customers",
    id: "id",
    pii: &["email", "password", "full_name", "addr", "recipient", "phone", "tel", "postal", "birth"],
};
const PAYMENTS: Payments = ORDERS.payments("payment_ref", "orders_settled_have_payment");
const VARIANTS: Variants = Variants { table: "product_variants", id: "id", product: "product_id" };
const RECORDS: RecordGuards = RecordGuards {
    order_delete: "orders_no_delete",
    order_update: &["orders_fixed", "orders_status_forward"],
    line_insert: "order_items_while_pending",
    line_update: "order_items_fixed",
    line_delete: "order_items_no_delete",
};
const SHIPMENT: ShipmentGuards = ShipmentGuards {
    order_update: "orders_shipped_in_full",
    line_insert: "order_items_shipped_in_full_on_insert",
    line_update: "order_items_shipped_in_full_on_update",
};

/// 中身の空なトリガー（名前だけを見る語のテストに使う）。
fn empty_trigger(name: &str, event: &str, table: &str) -> String {
    format!("CREATE TRIGGER {name} BEFORE {event} ON {table} FOR EACH ROW SET @x = 1;\n")
}

fn record_triggers() -> String {
    [
        empty_trigger("orders_no_delete", "DELETE", "orders"),
        empty_trigger("orders_fixed", "UPDATE", "orders"),
        empty_trigger("orders_status_forward", "UPDATE", "orders"),
        empty_trigger("order_items_while_pending", "INSERT", "order_items"),
        empty_trigger("order_items_fixed", "UPDATE", "order_items"),
        empty_trigger("order_items_no_delete", "DELETE", "order_items"),
    ]
    .concat()
}

fn shipment_triggers() -> String {
    [
        empty_trigger("orders_shipped_in_full", "UPDATE", "orders"),
        empty_trigger("order_items_shipped_in_full_on_insert", "INSERT", "order_items"),
        empty_trigger("order_items_shipped_in_full_on_update", "UPDATE", "order_items"),
    ]
    .concat()
}

fn server() -> Server {
    Server::attach(std::env::var(ENV).unwrap_or_else(|_| panic!("set {ENV} (run the tests with `make test`)")))
}

/// スキーマと代表データを当て、さらに extra を当てた DB の上で、各語の結果を返す。
fn verdicts(extra: &str, words: &[Word]) -> Vec<(String, Verdict)> {
    let server = server();
    let db = Scratch::create(&server, "ec").unwrap();
    db.db.run(shop::SCHEMA).unwrap();
    db.db.run(shop::DATA).unwrap();
    if !extra.is_empty() {
        db.db.run(extra).unwrap();
    }
    words.iter().map(|w| (w.name().to_owned(), w.holds(&db.db))).collect()
}

fn present_words() -> Vec<Word> {
    vec![
        CATALOG.prices_are_whole_minor_units(),
        CATALOG.stock_stays_above_its_floor(),
        ORDERS.amounts_are_whole_minor_units(),
        ORDERS.totals_match_lines(),
        ORDERS.are_keyed_by_request(),
        ORDERS.lines_refer_to(&CATALOG),
        ORDERS.lines_have_their_own_key(),
        ORDERS.status_is_named(),
        CUSTOMERS.pii_stays_here(),
        CUSTOMERS.are_referred_to_by(&ORDERS),
        CUSTOMERS.orders_outlive_them(&ORDERS),
        PAYMENTS.settled_orders_carry_a_payment(),
        PAYMENTS.payments_are_used_once(),
    ]
}

fn future_words() -> Vec<Word> {
    vec![
        CATALOG.prices_carry_currency("currency"),
        CATALOG.stock_allows_backorder("backorder_limit"),
        CATALOG.sold_as_variants(&VARIANTS),
        ORDERS.are_kept_as_records(&CUSTOMERS, &RECORDS),
        ORDERS.carry_currency("currency"),
        ORDERS.lines_share_the_currency("currency", "currency", "order_items_currency_on_insert"),
        ORDERS.taxes_match_rates("tax", "tax_rate_bp"),
        ORDERS.lines_name_variants("variant_id", &VARIANTS),
        ORDERS.lines_track_shipment("shipped_quantity"),
        ORDERS.shipped_orders_are_fully_shipped("shipped_quantity", "shipped", &SHIPMENT),
        CUSTOMERS.erasure_takes_their_orders(&ORDERS, "customers_erase_settled"),
    ]
}

/// 成り立たなかった語（確かめられなかったものも含む）。
fn failing(v: &[(String, Verdict)]) -> Vec<&str> {
    v.iter().filter(|(_, d)| !d.ok()).map(|(n, _)| n.as_str()).collect()
}

#[test]
fn the_present_words_hold_on_the_shop() {
    let v = verdicts("", &present_words());
    assert_eq!(failing(&v), Vec::<&str>::new(), "{v:?}");
}

#[test]
fn the_words_have_no_flaws() {
    let words = [present_words(), future_words()].concat();
    let flaws: Vec<String> = words.iter().flat_map(|w| w.fact().flaws()).collect();
    assert_eq!(flaws, Vec::<String>::new());
}

/// 未来の語は、今はまだ成り立たない。列や表がまだないことを理由に Fail し、Error（SQL の失敗）にはならない。
#[test]
fn the_future_words_fail_rather_than_error_before_their_columns_exist() {
    let v = verdicts("", &future_words());
    let not_failing: Vec<_> = v.iter().filter(|(_, d)| !matches!(d, Verdict::Fail(_))).collect();
    assert_eq!(not_failing, Vec::<&(String, Verdict)>::new());
}

#[test]
fn a_total_that_does_not_match_its_lines_is_found() {
    let v = verdicts("UPDATE orders SET total = total + 1 WHERE id = 2;", &[ORDERS.totals_match_lines()]);
    assert_eq!(failing(&v), ["order_totals_match_lines"]);
    assert!(v[0].1.problems()[0].contains("order_total_mismatch.sql: {id=2, total=1681, expected=1680}"), "{v:?}");
}

#[test]
fn pii_is_found_by_a_part_of_the_column_name() {
    let copies = "ALTER TABLE orders ADD COLUMN notify_email VARCHAR(254) NULL, ADD COLUMN Ship_To_Addr VARCHAR(500) NULL,\n\
                    ADD COLUMN recipient_name VARCHAR(100) NULL, ADD COLUMN contact_phone VARCHAR(20) NULL,\n\
                    ADD COLUMN ship_tel VARCHAR(20) NULL, ADD COLUMN postal_code CHAR(8) NULL, ADD COLUMN birth_date DATE NULL;\n\
                  CREATE TABLE invoices (id INT PRIMARY KEY, billing_email_hash CHAR(64), email_address VARCHAR(254));";
    let v = verdicts(copies, &[CUSTOMERS.pii_stays_here()]);
    let problems = v[0].1.problems().join("\n");
    for column in [
        "notify_email",
        "Ship_To_Addr",
        "recipient_name",
        "contact_phone",
        "ship_tel",
        "postal_code",
        "birth_date",
        "billing_email_hash",
        "email_address",
    ] {
        assert!(problems.contains(&format!("c={column}")), "{column}: {problems}");
    }
    // 商品の name や、ほかの未来が足す列（label・status・shipped_quantity など）は個人情報とみなさない
    assert!(!problems.contains("t=products"), "{problems}");
}

#[test]
fn keying_lines_by_product_closes_the_room_for_variants() {
    let word = ORDERS.lines_have_their_own_key();
    let pair = "ALTER TABLE order_items ADD UNIQUE KEY order_items_once (order_id, product_id);";
    assert_eq!(failing(&verdicts(pair, std::slice::from_ref(&word))), ["lines_have_their_own_key"]);
    // 組の一部（商品だけ）の一意索引でも、同じ商品を 2 行は入れられない
    let product = "DELETE FROM order_items WHERE (order_id, line_no) IN ((2, 2), (3, 1), (5, 1));\n\
                   ALTER TABLE order_items ADD UNIQUE KEY order_items_product_once (product_id);";
    assert_eq!(failing(&verdicts(product, &[word])), ["lines_have_their_own_key"]);
}

const CASCADE: &str = "ALTER TABLE orders DROP FOREIGN KEY orders_customer;\n\
     ALTER TABLE orders ADD CONSTRAINT orders_customer FOREIGN KEY (customer_id) REFERENCES customers (id) ON DELETE CASCADE;\n\
     ALTER TABLE order_items DROP FOREIGN KEY order_items_order;\n\
     ALTER TABLE order_items ADD CONSTRAINT order_items_order FOREIGN KEY (order_id) REFERENCES orders (id) ON DELETE CASCADE;\n";

#[test]
fn cascading_deletes_switch_from_outliving_to_erasure() {
    let words = [
        CUSTOMERS.erasure_takes_their_orders(&ORDERS, "customers_erase_settled"),
        CUSTOMERS.orders_outlive_them(&ORDERS),
    ];
    let guarded = format!("{CASCADE}{}", empty_trigger("customers_erase_settled", "DELETE", "customers"));
    assert_eq!(failing(&verdicts(&guarded, &words)), ["orders_outlive_customers"]);
    // 決着していない注文を守るトリガーがなければ、連鎖しても消去は成り立たない
    assert_eq!(
        failing(&verdicts(CASCADE, &words)),
        ["erasing_a_customer_takes_their_orders", "orders_outlive_customers"]
    );
}

#[test]
fn without_the_foreign_keys_neither_erasure_nor_outliving_holds() {
    let loose = format!(
        "ALTER TABLE order_items DROP FOREIGN KEY order_items_order;\n\
         ALTER TABLE orders DROP FOREIGN KEY orders_customer;\n{}",
        empty_trigger("customers_erase_settled", "DELETE", "customers")
    );
    let v = verdicts(
        &loose,
        &[
            CUSTOMERS.erasure_takes_their_orders(&ORDERS, "customers_erase_settled"),
            CUSTOMERS.orders_outlive_them(&ORDERS),
        ],
    );
    assert_eq!(failing(&v), ["erasing_a_customer_takes_their_orders", "orders_outlive_customers"]);
}

#[test]
fn records_are_not_kept_while_a_cascade_reaches_them() {
    let word = ORDERS.are_kept_as_records(&CUSTOMERS, &RECORDS);
    assert_eq!(failing(&verdicts(&record_triggers(), std::slice::from_ref(&word))), Vec::<&str>::new());
    let cascade = format!(
        "{}ALTER TABLE orders DROP FOREIGN KEY orders_customer;\n\
         ALTER TABLE orders ADD CONSTRAINT orders_customer FOREIGN KEY (customer_id) REFERENCES customers (id) ON DELETE CASCADE;",
        record_triggers()
    );
    assert_eq!(failing(&verdicts(&cascade, &[word])), ["orders_are_kept_as_records"]);
}

/// 守りは名前で見る。ほかの名前のトリガー（ほかの未来の通貨や発送の守り）があっても、守りがあることにはならない。
#[test]
fn records_are_kept_only_by_the_named_triggers() {
    let word = ORDERS.are_kept_as_records(&CUSTOMERS, &RECORDS);
    let others = [
        empty_trigger("o_d", "DELETE", "orders"),
        empty_trigger("o_u", "UPDATE", "orders"),
        empty_trigger("i_i", "INSERT", "order_items"),
        empty_trigger("i_u", "UPDATE", "order_items"),
        empty_trigger("i_d", "DELETE", "order_items"),
    ]
    .concat();
    let v = verdicts(&others, std::slice::from_ref(&word));
    assert_eq!(failing(&v), ["orders_are_kept_as_records"]);
    assert!(v[0].1.problems().join("\n").contains("missing_trigger=orders_no_delete"), "{v:?}");
    // 名前が合っても、表やイベントが違えば数えない
    let misplaced = record_triggers().replace("BEFORE DELETE ON orders", "BEFORE DELETE ON customers");
    assert_eq!(failing(&verdicts(&misplaced, &[word])), ["orders_are_kept_as_records"]);
}

#[test]
fn taxes_are_rounded_per_line_with_the_verified_function() {
    // 明細ごとに切り捨てる（146 + 93 = 239）。まとめてから切り捨てると 4400 × 3.33% + 1200 × 7.8% = 240.12 → 240。
    let add = "ALTER TABLE orders ADD COLUMN tax BIGINT UNSIGNED NOT NULL DEFAULT 0;\n\
               ALTER TABLE order_items ADD COLUMN tax_rate_bp SMALLINT UNSIGNED NOT NULL DEFAULT 1000;\n\
               UPDATE order_items SET tax_rate_bp = 333 WHERE order_id = 1 AND line_no = 1;\n\
               UPDATE order_items SET tax_rate_bp = 780 WHERE order_id = 1 AND line_no = 2;\n\
               UPDATE orders SET tax = 168 WHERE id = 2;\n\
               UPDATE orders SET tax = 180 WHERE id = 3;\n\
               UPDATE orders SET tax = 30 WHERE id = 4;\n\
               UPDATE orders SET tax = 180 WHERE id = 5;\n";
    let per_line = tax(2 * 2200, 333).unwrap() + tax(1200, 780).unwrap();
    assert_eq!(per_line, 239);
    let word = ORDERS.taxes_match_rates("tax", "tax_rate_bp");
    let ok = format!("{add}UPDATE orders SET tax = {per_line} WHERE id = 1;");
    assert_eq!(failing(&verdicts(&ok, std::slice::from_ref(&word))), Vec::<&str>::new());
    let order_level = format!("{add}UPDATE orders SET tax = 240 WHERE id = 1;");
    assert_eq!(failing(&verdicts(&order_level, std::slice::from_ref(&word))), ["order_taxes_match_rates"]);
    let over = format!(
        "{add}UPDATE orders SET tax = {per_line} WHERE id = 1;\nUPDATE order_items SET tax_rate_bp = 10001 WHERE order_id = 2 AND line_no = 1;"
    );
    assert_eq!(failing(&verdicts(&over, &[word])), ["order_taxes_match_rates"]);
}

#[test]
fn a_settled_order_without_a_named_guard_on_its_payment_is_found() {
    let word = PAYMENTS.settled_orders_carry_a_payment();
    assert_eq!(PAYMENTS.table(), "orders");
    let dropped = "ALTER TABLE orders DROP CHECK orders_settled_have_payment;";
    assert_eq!(failing(&verdicts(dropped, std::slice::from_ref(&word))), ["settled_orders_carry_a_payment"]);
    let hollow =
        format!("{dropped}\nALTER TABLE orders ADD CONSTRAINT orders_settled_have_payment CHECK (total >= 0);");
    assert_eq!(failing(&verdicts(&hollow, std::slice::from_ref(&word))), ["settled_orders_carry_a_payment"]);
    // 名前の一部が同じ別の列を条件にしても、参照の列を守ったことにはならない
    let lookalike = format!(
        "{dropped}\nALTER TABLE orders ADD COLUMN x_payment_ref VARCHAR(64) NULL;\n\
         UPDATE orders SET x_payment_ref = payment_ref;\n\
         ALTER TABLE orders ADD CONSTRAINT orders_settled_have_payment\n\
           CHECK (status IN ('pending', 'declined') OR x_payment_ref IS NOT NULL);"
    );
    assert_eq!(failing(&verdicts(&lookalike, &[word])), ["settled_orders_carry_a_payment"]);
}

#[test]
fn a_negative_stock_breaks_the_floor_even_when_the_check_is_named() {
    let hollow = "ALTER TABLE products DROP CHECK products_stock_nonnegative;\n\
                  ALTER TABLE products ADD CONSTRAINT products_stock_nonnegative CHECK (stock >= -10);\n\
                  UPDATE products SET stock = -1 WHERE id = 4;";
    assert_eq!(failing(&verdicts(hollow, &[CATALOG.stock_stays_above_its_floor()])), ["stock_stays_above_its_floor"]);
}

#[test]
fn a_backorder_limit_must_sit_beside_the_stock() {
    let word = CATALOG.stock_allows_backorder("backorder_limit");
    let add = "ALTER TABLE products ADD COLUMN backorder_limit INT NOT NULL DEFAULT 0;\n";
    assert_eq!(failing(&verdicts(add, std::slice::from_ref(&word))), Vec::<&str>::new());
    // 在庫がバリエーションの表へ移ると、商品の表の上限は在庫を縛らない
    let moved = format!(
        "{add}CREATE TABLE product_variants (id BIGINT UNSIGNED PRIMARY KEY, product_id BIGINT UNSIGNED NOT NULL, stock INT NOT NULL);"
    );
    assert_eq!(failing(&verdicts(&moved, &[word])), ["stock_allows_backorder"]);
}

#[test]
fn shipped_orders_are_checked_with_the_given_status() {
    let add = format!(
        "ALTER TABLE order_items ADD COLUMN shipped_quantity INT UNSIGNED NOT NULL DEFAULT 0;\n\
         UPDATE order_items SET shipped_quantity = quantity WHERE order_id = 1;\n{}",
        shipment_triggers()
    );
    let shipped = ORDERS.shipped_orders_are_fully_shipped("shipped_quantity", "shipped", &SHIPMENT);
    assert_eq!(failing(&verdicts(&add, std::slice::from_ref(&shipped))), Vec::<&str>::new());
    let partial = format!("{add}UPDATE order_items SET shipped_quantity = 1 WHERE order_id = 1 AND line_no = 1;");
    assert_eq!(failing(&verdicts(&partial, std::slice::from_ref(&shipped))), ["shipped_orders_are_fully_shipped"]);
    // 明細の側の守り（名前の付いたトリガー）がなければ成り立たない
    let unguarded = add.replace("order_items_shipped_in_full_on_update", "something_else");
    assert_eq!(failing(&verdicts(&unguarded, &[shipped])), ["shipped_orders_are_fully_shipped"]);
    // 状態の値を差し替えると、支払済み（何も発送していない）の注文が対象になる
    let paid = ORDERS.shipped_orders_are_fully_shipped("shipped_quantity", "paid", &SHIPMENT);
    assert_eq!(failing(&verdicts(&add, &[paid])), ["shipped_orders_are_fully_shipped"]);
}

#[test]
fn a_line_in_another_currency_is_found() {
    let add = format!(
        "ALTER TABLE products ADD COLUMN currency CHAR(3) NOT NULL DEFAULT 'JPY';\n\
         ALTER TABLE orders ADD COLUMN currency CHAR(3) NOT NULL DEFAULT 'JPY';\n\
         ALTER TABLE order_items ADD COLUMN currency CHAR(3) NOT NULL DEFAULT 'JPY';\n{}",
        empty_trigger("order_items_currency_on_insert", "INSERT", "order_items")
    );
    let word = ORDERS.lines_share_the_currency("currency", "currency", "order_items_currency_on_insert");
    assert_eq!(failing(&verdicts(&add, std::slice::from_ref(&word))), Vec::<&str>::new());
    // 明細は通貨を写して持つので、商品の通貨を後で変えても過去の注文は変わらない
    let repriced = format!("{add}UPDATE products SET currency = 'USD' WHERE id = 3;");
    assert_eq!(failing(&verdicts(&repriced, std::slice::from_ref(&word))), Vec::<&str>::new());
    let mixed = format!("{add}UPDATE order_items SET currency = 'USD' WHERE order_id = 2 AND line_no = 2;");
    assert_eq!(failing(&verdicts(&mixed, std::slice::from_ref(&word))), ["lines_share_the_order_currency"]);
    // 守りのトリガーは名前で見る
    let unguarded = add.replace("order_items_currency_on_insert", "something_else");
    assert_eq!(failing(&verdicts(&unguarded, &[word])), ["lines_share_the_order_currency"]);
}

#[test]
fn a_line_must_name_a_variant_of_its_own_product() {
    let add = "CREATE TABLE product_variants (id BIGINT UNSIGNED PRIMARY KEY, product_id BIGINT UNSIGNED NOT NULL,\n\
               UNIQUE KEY product_variants_of (id, product_id), FOREIGN KEY (product_id) REFERENCES products (id));\n\
               INSERT INTO product_variants SELECT id, id FROM products;\n\
               ALTER TABLE order_items ADD COLUMN variant_id BIGINT UNSIGNED NULL;\n\
               UPDATE order_items SET variant_id = product_id;\n";
    let word = ORDERS.lines_name_variants("variant_id", &VARIANTS);
    // 組の外部キーがなければ成り立たない
    assert_eq!(failing(&verdicts(add, std::slice::from_ref(&word))), ["lines_name_variants"]);
    let keyed = format!(
        "{add}ALTER TABLE order_items ADD CONSTRAINT order_items_variant FOREIGN KEY (variant_id, product_id) REFERENCES product_variants (id, product_id);"
    );
    assert_eq!(failing(&verdicts(&keyed, std::slice::from_ref(&word))), Vec::<&str>::new());
    let single = format!(
        "{add}ALTER TABLE order_items ADD CONSTRAINT order_items_variant FOREIGN KEY (variant_id) REFERENCES product_variants (id);\n\
         UPDATE order_items SET variant_id = 1 WHERE order_id = 2 AND line_no = 1;"
    );
    assert_eq!(failing(&verdicts(&single, &[word])), ["lines_name_variants"]);
}

#[test]
fn all_of_stops_at_the_first_word_that_does_not_hold() {
    let totals = ORDERS.totals_match_lines();
    let currency = CATALOG.prices_carry_currency("currency");
    let taxes = ORDERS.taxes_match_rates("tax", "tax_rate_bp");
    let here = all_of("multi_currency_is_here", "通貨と税がある", &[&totals, &currency, &taxes]);
    let v = verdicts("", std::slice::from_ref(&here));
    // 合計は成り立ち、通貨の列がないところで止まる（税の列がないことは言わない）
    assert_eq!(
        v[0].1.problems(),
        ["multi_currency_is_here: prices_carry_currency: no column products.currency"],
        "{v:?}"
    );
}

/// フレームワークのソース（`src/*.rs`）だけを見る。`src/bin/with-mysql.rs` はテストの道具で、層2ではない。
#[test]
fn the_framework_holds_no_sql_and_uses_only_layer_1() {
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let allowed = ["ai_yagni_data_sdk::vocab", "ai_yagni_data_ec_verified", "crate"];
    let mut files: Vec<_> = std::fs::read_dir(&src)
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|x| x == "rs"))
        .collect();
    files.sort();
    assert!(files.len() >= 5, "{files:?}");
    let problems: Vec<String> =
        files.iter().flat_map(|p| lint::file(p, &p.file_name().unwrap().to_string_lossy(), &allowed)).collect();
    assert_eq!(problems, Vec::<String>::new());
}

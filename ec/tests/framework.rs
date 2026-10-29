//! フレームワークの語が、EC の形のスキーマ（tests/fixtures/shop）の上で成り立ち、壊れた形では成り立たないこと。
//! 層2の規則（SQL を文字列で持たない、SDK の層1だけを使う）もここで確かめる。
//!
//! MySQL は `AI_YAGNI_MYSQL` のサーバーを使う（`make test` が `with-mysql cargo test` で立てる）。
use std::path::Path;

use ai_yagni_data_ec::{Catalog, Customers, Lines, Orders, Payments, Variants};
use ai_yagni_data_sdk::lint;
use ai_yagni_data_sdk::mysql::{ENV, Scratch, Server};
use ai_yagni_data_sdk::vocab::Word;

const SCHEMA: &str = include_str!("fixtures/shop/schema.sql");
const DATA: &str = include_str!("fixtures/shop/0001_shop.sql");

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
const CUSTOMERS: Customers =
    Customers { table: "customers", id: "id", pii: &["email", "password_hash", "full_name", "address"] };
const PAYMENTS: Payments = Payments { table: "orders", reference: "payment_ref" };
const VARIANTS: Variants = Variants { table: "product_variants", id: "id", product: "product_id" };

fn server() -> Server {
    Server::attach(std::env::var(ENV).unwrap_or_else(|_| panic!("set {ENV} (run the tests with `make test`)")))
}

/// スキーマと代表データを当て、さらに extra を当てた DB の上で、各語の理由（成り立てば空）を返す。
fn verdicts(extra: &str, words: &[Word]) -> Vec<(String, Vec<String>)> {
    let server = server();
    let db = Scratch::create(&server, "ec").unwrap();
    db.db.run(SCHEMA).unwrap();
    db.db.run(DATA).unwrap();
    if !extra.is_empty() {
        db.db.run(extra).unwrap();
    }
    words.iter().map(|w| (w.name().to_owned(), w.holds(&db.db).0)).collect()
}

fn present_words() -> Vec<Word> {
    vec![
        CATALOG.prices_are_whole_minor_units(),
        CATALOG.stock_floor_is_named(),
        ORDERS.amounts_are_whole_minor_units(),
        ORDERS.lines_copy_the_price(),
        ORDERS.totals_match_lines(),
        ORDERS.are_keyed_by_request(),
        ORDERS.lines_refer_to(&CATALOG),
        ORDERS.lines_have_their_own_key(),
        ORDERS.status_is_named(),
        CUSTOMERS.pii_stays_here(),
        CUSTOMERS.are_referred_to_by(&ORDERS),
        CUSTOMERS.orders_outlive_them(&ORDERS),
        PAYMENTS.orders_carry_a_payment(),
        PAYMENTS.payments_are_used_once(),
    ]
}

fn failing(v: &[(String, Vec<String>)]) -> Vec<&str> {
    v.iter().filter(|(_, d)| !d.is_empty()).map(|(n, _)| n.as_str()).collect()
}

#[test]
fn the_present_words_hold_on_the_shop() {
    let v = verdicts("", &present_words());
    assert_eq!(failing(&v), Vec::<&str>::new(), "{v:?}");
}

#[test]
fn the_future_words_do_not_hold_yet() {
    let words = [
        CATALOG.prices_carry_currency("currency"),
        CATALOG.stock_allows_backorder("backorder_limit"),
        CATALOG.sold_as_variants(&VARIANTS),
        ORDERS.are_kept_as_records(),
        ORDERS.lines_track_shipment("shipped_quantity"),
        CUSTOMERS.erasure_takes_their_orders(&ORDERS),
    ];
    let v = verdicts("", &words);
    assert_eq!(failing(&v).len(), words.len(), "{v:?}");
}

#[test]
fn a_total_that_does_not_match_its_lines_is_found() {
    let v = verdicts("UPDATE orders SET total = total + 1 WHERE id = 2;", &[ORDERS.totals_match_lines()]);
    assert_eq!(failing(&v), ["order_totals_match_lines"]);
    assert!(v[0].1[0].contains("order_total_mismatch.sql: {id=2, total=1681, expected=1680}"), "{v:?}");
}

#[test]
fn copying_an_email_into_orders_breaks_pii_stays_on_customers() {
    let v = verdicts("ALTER TABLE orders ADD COLUMN notify_email VARCHAR(254) NULL;", &[CUSTOMERS.pii_stays_here()]);
    assert_eq!(v[0].1, ["pii_stays_on_customers: orders.notify_email is outside customers"]);
}

#[test]
fn keying_lines_by_product_closes_the_room_for_variants() {
    let v = verdicts(
        "ALTER TABLE order_items ADD UNIQUE KEY order_items_once (order_id, product_id);",
        &[ORDERS.lines_have_their_own_key()],
    );
    assert_eq!(failing(&v), ["lines_have_their_own_key"]);
}

#[test]
fn cascading_deletes_switch_from_outliving_to_erasure() {
    let cascade = "ALTER TABLE orders DROP FOREIGN KEY orders_customer;\n\
                   ALTER TABLE orders ADD CONSTRAINT orders_customer FOREIGN KEY (customer_id) REFERENCES customers (id) ON DELETE CASCADE;\n\
                   ALTER TABLE order_items DROP FOREIGN KEY order_items_order;\n\
                   ALTER TABLE order_items ADD CONSTRAINT order_items_order FOREIGN KEY (order_id) REFERENCES orders (id) ON DELETE CASCADE;";
    let v = verdicts(cascade, &[CUSTOMERS.erasure_takes_their_orders(&ORDERS), CUSTOMERS.orders_outlive_them(&ORDERS)]);
    assert_eq!(failing(&v), ["orders_outlive_customers"]);
}

#[test]
fn taxes_are_checked_with_the_same_rounding_as_the_verified_function() {
    let add = "ALTER TABLE orders ADD COLUMN tax BIGINT UNSIGNED NOT NULL DEFAULT 0;\n\
               ALTER TABLE order_items ADD COLUMN tax_rate_bp SMALLINT UNSIGNED NOT NULL DEFAULT 0;\n\
               UPDATE order_items SET tax_rate_bp = 1000;\n";
    let expected = ai_yagni_data_ec::tax(5600, 1000).unwrap();
    let word = ORDERS.taxes_match_rates("tax", "tax_rate_bp");
    let ok = format!("{add}UPDATE orders SET tax = {expected} WHERE id = 1; UPDATE orders SET tax = 168 WHERE id = 2;");
    assert_eq!(failing(&verdicts(&ok, std::slice::from_ref(&word))), Vec::<&str>::new());
    let off = format!("{add}UPDATE orders SET tax = {} WHERE id = 1;", expected + 1);
    assert_eq!(failing(&verdicts(&off, &[word])), ["order_taxes_match_rates"]);
}

#[test]
fn the_framework_holds_no_sql_and_uses_only_layer_1() {
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let allowed = ["ai_yagni_data_sdk::vocab", "ai_yagni_data_ec_verified", "crate"];
    assert_eq!(lint::dir(&src, &allowed), Vec::<String>::new());
}

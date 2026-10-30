//! 注文と明細。
use ai_yagni_data_sdk::vocab::{
    Event, Kind, OnDelete, SqlFile, Word, all, check_exists, column_is, define, foreign_key, guarded_against, not,
    references_to, shape, sql_empty, unique_on,
};

use crate::catalog::{Catalog, Variants};
use crate::customers::Customers;
use crate::in_order;

const ORDER_TOTAL_MISMATCH: SqlFile =
    SqlFile::new("order_total_mismatch.sql", include_str!("sql/order_total_mismatch.sql"));
const TAX_MISMATCH: SqlFile = SqlFile::new("tax_mismatch.sql", include_str!("sql/tax_mismatch.sql"));
const TAX_RATE_OUT_OF_RANGE: SqlFile =
    SqlFile::new("tax_rate_out_of_range.sql", include_str!("sql/tax_rate_out_of_range.sql"));
const SHIPPED_BUT_NOT_FULLY: SqlFile =
    SqlFile::new("shipped_but_not_fully.sql", include_str!("sql/shipped_but_not_fully.sql"));
const LINE_CURRENCY_MISMATCH: SqlFile =
    SqlFile::new("line_currency_mismatch.sql", include_str!("sql/line_currency_mismatch.sql"));
const LINE_VARIANT_OF_ANOTHER_PRODUCT: SqlFile =
    SqlFile::new("line_variant_of_another_product.sql", include_str!("sql/line_variant_of_another_product.sql"));

/// 注文の明細の表。
#[derive(Debug, Clone, Copy)]
pub struct Lines {
    pub table: &'static str,
    pub order: &'static str,
    pub line_no: &'static str,
    pub product: &'static str,
    pub quantity: &'static str,
    pub unit_price: &'static str,
}

/// 注文の表と、その明細。
#[derive(Debug, Clone, Copy)]
pub struct Orders {
    pub table: &'static str,
    pub id: &'static str,
    pub customer: &'static str,
    pub request_key: &'static str,
    pub total: &'static str,
    pub status: &'static str,
    /// 状態の CHECK 制約の名前。
    pub status_check: &'static str,
    pub lines: Lines,
}

impl Orders {
    /// SQL ファイルに差し込む、注文と明細をつなぐ名前（各語は、これに自分の SQL が使う名前を足す）。
    fn joined(&self) -> Vec<(&str, &str)> {
        vec![
            ("orders", self.table),
            ("order_id", self.id),
            ("lines", self.lines.table),
            ("line_order", self.lines.order),
        ]
    }

    pub fn amounts_are_whole_minor_units(&self) -> Word {
        define(
            "order_amounts_are_whole_minor_units",
            "注文の合計と、明細が持つ注文した時点の単価の写しは、通貨の最小単位の整数で持つ（商品の価格が後で変わっても、注文は変わらない）",
            all([
                column_is(self.table, self.total, shape().kind(Kind::Integer).not_null()),
                column_is(self.lines.table, self.lines.unit_price, shape().kind(Kind::Integer).not_null()),
            ]),
        )
    }

    pub fn totals_match_lines(&self) -> Word {
        let mut names = self.joined();
        names.extend([("total", self.total), ("quantity", self.lines.quantity), ("unit_price", self.lines.unit_price)]);
        define(
            "order_totals_match_lines",
            "注文の合計は、明細の数量 × 単価の和と一致する（合計を明細から作り直せる）",
            sql_empty(ORDER_TOTAL_MISMATCH, &names),
        )
    }

    pub fn are_keyed_by_request(&self) -> Word {
        define(
            "orders_are_keyed_by_request",
            "注文は顧客ごとの依頼の冪等キーで一意（同じ依頼を 2 度送っても注文は 1 つ）。キーは省けない",
            all([
                unique_on(self.table, &[self.customer, self.request_key]),
                column_is(self.table, self.customer, shape().not_null()),
                column_is(self.table, self.request_key, shape().not_null()),
            ]),
        )
    }

    pub fn lines_refer_to(&self, catalog: &Catalog) -> Word {
        define(
            "lines_refer_to_products",
            "明細は商品を ID で指す",
            foreign_key(self.lines.table, &[self.lines.product], catalog.table, &[catalog.id]),
        )
    }

    pub fn lines_have_their_own_key(&self) -> Word {
        define(
            "lines_have_their_own_key",
            "明細は注文の中の行番号で区別する（同じ商品を 1 つの注文に何行でも入れられる）",
            all([
                unique_on(self.lines.table, &[self.lines.order, self.lines.line_no]),
                not(unique_on(self.lines.table, &[self.lines.order, self.lines.product])),
            ]),
        )
    }

    /// CHECK 制約の中身（どの状態を許すか）は見ない。候補を足した後の中身は、未来の rejects で確かめる。
    pub fn status_is_named(&self) -> Word {
        define(
            "order_status_is_named",
            "注文の状態の候補を守る CHECK 制約に名前が付いている（未来の差分がその名前で付け替えられる。中身は見ない）",
            check_exists(self.table, self.status_check),
        )
    }

    /// トリガーはあることしか見ない。何を拒むかは、未来の rejects で確かめる。
    /// 外部キーの連鎖した削除（ON DELETE CASCADE）はトリガーを起こさないので、注文と顧客を指す外部キーは連鎖させない。
    pub fn are_kept_as_records(&self, customers: &Customers) -> Word {
        define(
            "orders_are_kept_as_records",
            "注文と明細は消せず、注文の金額と相手、明細の中身（何をいくつ幾らで）は書き換えられない（売上の記録として残す）。顧客を消しても連鎖して消えない",
            all([
                guarded_against(self.table, Event::Delete),
                guarded_against(self.table, Event::Update),
                guarded_against(self.lines.table, Event::Update),
                guarded_against(self.lines.table, Event::Delete),
                references_to(self.table, &[OnDelete::Restrict, OnDelete::NoAction]),
                references_to(customers.table, &[OnDelete::Restrict, OnDelete::NoAction]),
            ]),
        )
    }

    pub fn carry_currency(&self, currency: &str) -> Word {
        define(
            "orders_carry_currency",
            "注文が通貨を持つ",
            column_is(self.table, currency, shape().kind(Kind::String).not_null()),
        )
    }

    /// currency は注文の通貨の列、product_currency は商品の通貨の列。
    /// 書き込みを守るのは明細の INSERT の前のトリガー（あることだけを見る）。今の行は SQL で確かめる。
    pub fn lines_share_the_currency(&self, currency: &str, catalog: &Catalog, product_currency: &str) -> Word {
        let mut names = self.joined();
        names.extend([
            ("line_product", self.lines.product),
            ("order_currency", currency),
            ("products", catalog.table),
            ("product_id", catalog.id),
            ("product_currency", product_currency),
        ]);
        define(
            "lines_share_the_order_currency",
            "明細の商品の通貨は、注文の通貨と同じ（1 つの注文に通貨を混ぜない）",
            in_order(vec![
                column_is(self.table, currency, shape().not_null()),
                column_is(catalog.table, product_currency, shape().not_null()),
                guarded_against(self.lines.table, Event::Insert),
                sql_empty(LINE_CURRENCY_MISMATCH, &names),
            ]),
        )
    }

    /// tax は注文の税額の列、tax_rate は明細の税率（basis point）の列。
    pub fn taxes_match_rates(&self, tax: &str, tax_rate: &str) -> Word {
        let mut names = self.joined();
        names.extend([
            ("quantity", self.lines.quantity),
            ("unit_price", self.lines.unit_price),
            ("tax", tax),
            ("tax_rate", tax_rate),
        ]);
        let rates = [("lines", self.lines.table), ("line_order", self.lines.order), ("tax_rate", tax_rate)];
        define(
            "order_taxes_match_rates",
            "税率は 0〜100%。注文の税は、明細ごとの数量 × 単価 × 税率 ÷ 10000 の切り捨ての和と一致する",
            in_order(vec![
                column_is(self.table, tax, shape().kind(Kind::Integer).not_null()),
                column_is(self.lines.table, tax_rate, shape().kind(Kind::Integer).not_null()),
                sql_empty(TAX_RATE_OUT_OF_RANGE, &rates),
                sql_empty(TAX_MISMATCH, &names),
            ]),
        )
    }

    /// variant は明細のバリエーションの列。外部キーは（バリエーション, 商品）の組で指す。
    pub fn lines_name_variants(&self, variant: &str, variants: &Variants) -> Word {
        let names = [
            ("lines", self.lines.table),
            ("line_order", self.lines.order),
            ("line_product", self.lines.product),
            ("variant", variant),
            ("variants", variants.table),
            ("variant_id", variants.id),
            ("variant_product", variants.product),
        ];
        define(
            "lines_name_variants",
            "明細は商品のバリエーションを ID で指せる。指すなら、その明細の商品のバリエーション",
            in_order(vec![
                foreign_key(
                    self.lines.table,
                    &[variant, self.lines.product],
                    variants.table,
                    &[variants.id, variants.product],
                ),
                sql_empty(LINE_VARIANT_OF_ANOTHER_PRODUCT, &names),
            ]),
        )
    }

    pub fn lines_track_shipment(&self, shipped: &str) -> Word {
        define(
            "lines_track_shipment",
            "明細ごとに発送した数を持つ（一部だけ発送できる）",
            column_is(self.lines.table, shipped, shape().kind(Kind::Integer).not_null()),
        )
    }

    /// shipped は明細の発送した数の列、shipped_status は発送済みを表す状態の値。
    /// 値は SQL の文字列として差し込むので、英数字と `_` だけでできていなければならない（`shipped` など）。
    pub fn shipped_orders_are_fully_shipped(&self, shipped: &str, shipped_status: &str) -> Word {
        let mut names = self.joined();
        names.extend([
            ("quantity", self.lines.quantity),
            ("status", self.status),
            ("shipped", shipped),
            ("shipped_status", shipped_status),
        ]);
        define(
            "shipped_orders_are_fully_shipped",
            "発送済みの注文は、どの明細も注文した数をすべて発送している",
            in_order(vec![
                column_is(self.lines.table, shipped, shape().kind(Kind::Integer).not_null()),
                sql_empty(SHIPPED_BUT_NOT_FULLY, &names),
            ]),
        )
    }
}

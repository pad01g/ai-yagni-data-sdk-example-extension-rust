//! 注文と明細。
use ai_yagni_data_sdk::vocab::{
    Event, Kind, SqlFile, Word, all, check_exists, column_is, define, foreign_key, guarded_against, not, shape,
    sql_empty, unique_on,
};

use crate::catalog::{Catalog, Variants};

const ORDER_TOTAL_MISMATCH: SqlFile =
    SqlFile::new("order_total_mismatch.sql", include_str!("sql/order_total_mismatch.sql"));
const TAX_MISMATCH: SqlFile = SqlFile::new("tax_mismatch.sql", include_str!("sql/tax_mismatch.sql"));
const SHIPPED_BUT_NOT_FULLY: SqlFile =
    SqlFile::new("shipped_but_not_fully.sql", include_str!("sql/shipped_but_not_fully.sql"));

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
    fn names(&self) -> [(&'static str, &'static str); 6] {
        [
            ("orders", self.table),
            ("order_id", self.id),
            ("total", self.total),
            ("lines", self.lines.table),
            ("line_order", self.lines.order),
            ("quantity", self.lines.quantity),
        ]
    }

    pub fn amounts_are_whole_minor_units(&self) -> Word {
        define(
            "order_amounts_are_whole_minor_units",
            "注文の合計と明細の単価は、通貨の最小単位の整数で持つ",
            all([
                column_is(self.table, self.total, shape().kind(Kind::Integer).not_null()),
                column_is(self.lines.table, self.lines.unit_price, shape().kind(Kind::Integer).not_null()),
            ]),
        )
    }

    pub fn lines_copy_the_price(&self) -> Word {
        define(
            "lines_copy_the_price",
            "明細は注文した時点の単価の写しを持つ（商品の価格が後で変わっても、注文は変わらない）",
            column_is(self.lines.table, self.lines.unit_price, shape().not_null()),
        )
    }

    pub fn totals_match_lines(&self) -> Word {
        let mut names = self.names().to_vec();
        names.push(("unit_price", self.lines.unit_price));
        define(
            "order_totals_match_lines",
            "注文の合計は、明細の数量 × 単価の和と一致する（合計を明細から作り直せる）",
            sql_empty(ORDER_TOTAL_MISMATCH, &names),
        )
    }

    pub fn are_keyed_by_request(&self) -> Word {
        define(
            "orders_are_keyed_by_request",
            "注文は顧客ごとの依頼の冪等キーで一意（同じ依頼を 2 度送っても注文は 1 つ）",
            unique_on(self.table, &[self.customer, self.request_key]),
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

    pub fn status_is_named(&self) -> Word {
        define(
            "order_status_is_named",
            "注文の状態の候補は名前の付いた CHECK 制約で守られている（未来の差分がその名前で候補を足せる）",
            check_exists(self.table, self.status_check),
        )
    }

    pub fn are_kept_as_records(&self) -> Word {
        define(
            "orders_are_kept_as_records",
            "注文と明細は消せず、明細は書き換えられない（売上の記録として残す）",
            all([
                guarded_against(self.table, Event::Delete),
                guarded_against(self.lines.table, Event::Update),
                guarded_against(self.lines.table, Event::Delete),
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

    /// tax は注文の税額の列、tax_rate は明細の税率（basis point）の列。
    pub fn taxes_match_rates(&self, tax: &'static str, tax_rate: &'static str) -> Word {
        let mut names = self.names().to_vec();
        names.extend([("unit_price", self.lines.unit_price), ("tax", tax), ("tax_rate", tax_rate)]);
        define(
            "order_taxes_match_rates",
            "注文の税は、明細ごとの数量 × 単価 × 税率 ÷ 10000 の切り捨ての和と一致する",
            sql_empty(TAX_MISMATCH, &names),
        )
    }

    pub fn lines_name_variants(&self, variant: &str, variants: &Variants) -> Word {
        define(
            "lines_name_variants",
            "明細は商品のバリエーションを ID で指せる",
            foreign_key(self.lines.table, &[variant], variants.table, &[variants.id]),
        )
    }

    pub fn lines_track_shipment(&self, shipped: &str) -> Word {
        define(
            "lines_track_shipment",
            "明細ごとに発送した数を持つ（一部だけ発送できる）",
            column_is(self.lines.table, shipped, shape().kind(Kind::Integer).not_null()),
        )
    }

    pub fn shipped_orders_are_fully_shipped(&self, shipped: &'static str) -> Word {
        let names = [
            ("orders", self.table),
            ("order_id", self.id),
            ("status", self.status),
            ("lines", self.lines.table),
            ("line_order", self.lines.order),
            ("quantity", self.lines.quantity),
            ("shipped", shipped),
        ];
        define(
            "shipped_orders_are_fully_shipped",
            "発送済みの注文は、どの明細も注文した数をすべて発送している",
            sql_empty(SHIPPED_BUT_NOT_FULLY, &names),
        )
    }
}

//! 商品と在庫。
use ai_yagni_data_sdk::vocab::{
    Kind, SqlFile, Word, check_exists, column_is, columns_named_only_in, define, foreign_key, seq, shape, sql_empty,
};

const PRODUCT_WITHOUT_VARIANT: SqlFile =
    SqlFile::new("product_without_variant.sql", include_str!("sql/product_without_variant.sql"));
const STOCK_BELOW_ZERO: SqlFile = SqlFile::new("stock_below_zero.sql", include_str!("sql/stock_below_zero.sql"));

/// 商品の表。
#[derive(Debug, Clone, Copy)]
pub struct Catalog {
    pub table: &'static str,
    pub id: &'static str,
    pub price: &'static str,
    pub stock: &'static str,
    /// 在庫の下限（0 以上）の CHECK 制約の名前。
    pub stock_floor: &'static str,
}

/// 商品のバリエーション（サイズ・色）の表。
#[derive(Debug, Clone, Copy)]
pub struct Variants {
    pub table: &'static str,
    pub id: &'static str,
    pub product: &'static str,
}

impl Catalog {
    pub fn prices_are_whole_minor_units(&self) -> Word {
        define(
            "prices_are_whole_minor_units",
            "商品の価格は通貨の最小単位の整数で持つ（小数の端数を持たない。通貨を足しても最小単位の整数で表せる）",
            column_is(self.table, self.price, shape().kind(Kind::Integer).not_null()),
        )
    }

    /// 名前の付いた CHECK 制約があること（構造）と、今の行に負の在庫がないこと（データ）の両方を見る。
    /// 制約の中身（負の在庫を書き込めないこと）は見ない。それは今の probe と未来の rejects で確かめる。
    pub fn stock_stays_above_its_floor(&self) -> Word {
        define(
            "stock_stays_above_its_floor",
            "在庫の下限は名前の付いた CHECK 制約にあり（未来の差分がその名前で外して付け替えられる）、今の行にも在庫が 0 を下回る商品はない",
            seq([
                check_exists(self.table, self.stock_floor),
                sql_empty(
                    STOCK_BELOW_ZERO,
                    &[("products", self.table), ("product_id", self.id), ("stock", self.stock)],
                ),
            ]),
        )
    }

    pub fn prices_carry_currency(&self, currency: &str) -> Word {
        define(
            "prices_carry_currency",
            "商品の価格が通貨を持つ",
            column_is(self.table, currency, shape().kind(Kind::String).not_null()),
        )
    }

    /// 上限は在庫と同じ表になければ意味がない。在庫の列（`stock` か `_stock` で終わる名前）が商品の表の外にあれば
    /// （在庫をバリエーションへ移したなど）、成り立たない。
    pub fn stock_allows_backorder(&self, limit: &str) -> Word {
        define(
            "stock_allows_backorder",
            "商品ごとに、在庫を超えて受けられる予約の上限がある。在庫は商品の表にだけある（上限と在庫が同じ行にある。在庫をほかの表へ移すなら、上限も一緒に移す）",
            seq([
                column_is(self.table, limit, shape().kind(Kind::Integer).not_null()),
                columns_named_only_in(&[self.stock], &[self.table]),
            ]),
        )
    }

    pub fn sold_as_variants(&self, v: &Variants) -> Word {
        define(
            "sold_as_variants",
            "どの商品も、少なくとも 1 つのバリエーションとして売る",
            seq([
                foreign_key(v.table, &[v.product], self.table, &[self.id]),
                sql_empty(
                    PRODUCT_WITHOUT_VARIANT,
                    &[
                        ("products", self.table),
                        ("product_id", self.id),
                        ("variants", v.table),
                        ("variant_product", v.product),
                    ],
                ),
            ]),
        )
    }
}

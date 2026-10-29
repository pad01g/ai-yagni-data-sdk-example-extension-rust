//! 商品と在庫。
use ai_yagni_data_sdk::vocab::{
    Kind, SqlFile, Word, all, check_exists, column_is, define, foreign_key, shape, sql_empty,
};

const PRODUCT_WITHOUT_VARIANT: SqlFile =
    SqlFile::new("product_without_variant.sql", include_str!("sql/product_without_variant.sql"));

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

    pub fn stock_floor_is_named(&self) -> Word {
        define(
            "stock_floor_is_named",
            "在庫の下限は名前の付いた CHECK 制約で守られている（未来の差分がその名前で外して付け替えられる）",
            check_exists(self.table, self.stock_floor),
        )
    }

    pub fn prices_carry_currency(&self, currency: &str) -> Word {
        define(
            "prices_carry_currency",
            "商品の価格が通貨を持つ",
            column_is(self.table, currency, shape().kind(Kind::String).not_null()),
        )
    }

    pub fn stock_allows_backorder(&self, limit: &str) -> Word {
        define(
            "stock_allows_backorder",
            "商品ごとに、在庫を超えて受けられる予約の上限がある",
            column_is(self.table, limit, shape().kind(Kind::Integer).not_null()),
        )
    }

    pub fn sold_as_variants(&self, v: &Variants) -> Word {
        define(
            "sold_as_variants",
            "どの商品も、少なくとも 1 つのバリエーションとして売る",
            all([
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

//! 決済。決済サービスの参照（決済 ID）を注文に持つ。
use ai_yagni_data_sdk::vocab::{Kind, Word, all, column_is, define, shape, unique_on};

/// 決済の参照を持つ表（ふつうは注文の表）と、その列。
#[derive(Debug, Clone, Copy)]
pub struct Payments {
    pub table: &'static str,
    pub reference: &'static str,
}

impl Payments {
    pub fn orders_carry_a_payment(&self) -> Word {
        define(
            "orders_carry_a_payment",
            "注文はどれも決済サービスの参照を持つ（決済が通った注文だけを書く）",
            column_is(self.table, self.reference, shape().kind(Kind::String).not_null()),
        )
    }

    pub fn payments_are_used_once(&self) -> Word {
        define(
            "payments_are_used_once",
            "1 つの決済は 1 つの注文にしか使わない",
            all([column_is(self.table, self.reference, shape().not_null()), unique_on(self.table, &[self.reference])]),
        )
    }
}

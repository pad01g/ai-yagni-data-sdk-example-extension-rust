//! 決済。決済サービスの参照（決済 ID）を注文に持つ。
//!
//! 決済の前に注文を書き（在庫を引き当てた時点）、決済が決着してから参照を書く形を許す。
//! 参照を持たなくてよい状態（決済の前、拒否された）はプロジェクトごとに違うので、状態の一覧はこの構成子に渡さない。
//! 代わりに、参照を必ず持つ条件を名前の付いた CHECK 制約に書き、その制約が参照の列を NULL でないことを条件に含むことを確かめる。
//! 条件の意味（どの状態で持ち、どの状態で持たないか）は、利用者のプロジェクトの今の probe で確かめる。
use ai_yagni_data_sdk::vocab::{
    Kind, SqlFile, Word, check_exists, column_is, define, seq, shape, sql_empty, unique_on,
};

const CHECK_WITHOUT_REFERENCE: SqlFile =
    SqlFile::new("check_without_reference.sql", include_str!("sql/check_without_reference.sql"));

/// 決済の参照を持つ注文の表と、その列。[`crate::Orders::payments`] で作る（表の名前を 2 度書かない）。
#[derive(Debug, Clone, Copy)]
pub struct Payments {
    table: &'static str,
    pub reference: &'static str,
    /// 「決着した注文は参照を持つ」CHECK 制約の名前（`status IN ('pending', 'declined') OR payment_ref IS NOT NULL` など）。
    pub settled_check: &'static str,
}

impl Payments {
    pub(crate) const fn of(table: &'static str, reference: &'static str, settled_check: &'static str) -> Self {
        Self { table, reference, settled_check }
    }

    /// 参照を持つ表（注文の表）。
    pub const fn table(&self) -> &'static str {
        self.table
    }

    /// 条件の文に、参照の列が `` `payment_ref` is not null `` の形（逆引用符で区切った識別子）で現れることを見る。
    /// 名前の一部が同じ別の列（`x_payment_ref`）では成り立たない。条件の意味までは見ない。
    pub fn settled_orders_carry_a_payment(&self) -> Word {
        define(
            "settled_orders_carry_a_payment",
            "決済が通った注文は決済サービスの参照を持つ（参照の列は文字列で、それを NULL でないことを条件に含む名前の付いた CHECK 制約が守る。条件の意味は今の probe で確かめる）",
            seq([
                column_is(self.table, self.reference, shape().kind(Kind::String)),
                check_exists(self.table, self.settled_check),
                sql_empty(
                    CHECK_WITHOUT_REFERENCE,
                    &[("table", self.table), ("check", self.settled_check), ("reference", self.reference)],
                ),
            ]),
        )
    }

    /// 一意索引は NULL を何行でも許すので、参照をまだ持たない注文はいくつあってもよい。
    pub fn payments_are_used_once(&self) -> Word {
        define(
            "payments_are_used_once",
            "1 つの決済は 1 つの注文にしか使わない",
            unique_on(self.table, &[self.reference]),
        )
    }
}

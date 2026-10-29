//! 顧客と、顧客を指す注文。
use ai_yagni_data_sdk::vocab::{OnDelete, Word, all, columns_named_only_in, define, foreign_key, references_to};

use crate::orders::Orders;

/// 顧客の表。pii は個人情報の列の名前（`email` なら `to_email` のような列も個人情報とみなす）。
#[derive(Debug, Clone, Copy)]
pub struct Customers {
    pub table: &'static str,
    pub id: &'static str,
    pub pii: &'static [&'static str],
}

impl Customers {
    pub fn pii_stays_here(&self) -> Word {
        define(
            "pii_stays_on_customers",
            "個人情報（メール・パスワードのハッシュ・氏名・住所）は顧客の表にだけある（注文などに写さない）",
            columns_named_only_in(self.pii, &[self.table]),
        )
    }

    pub fn are_referred_to_by(&self, orders: &Orders) -> Word {
        define(
            "orders_refer_to_customers",
            "注文は顧客を ID で指す（メールや氏名で指さない）",
            foreign_key(orders.table, &[orders.customer], self.table, &[self.id]),
        )
    }

    pub fn erasure_takes_their_orders(&self, orders: &Orders) -> Word {
        define(
            "erasing_a_customer_takes_their_orders",
            "顧客の行を消すと、その顧客の注文と明細も消える（個人と結び付く記録を残さない）",
            all([references_to(self.table, &[OnDelete::Cascade]), references_to(orders.table, &[OnDelete::Cascade])]),
        )
    }

    pub fn orders_outlive_them(&self, orders: &Orders) -> Word {
        define(
            "orders_outlive_customers",
            "顧客の行を消しても注文は道連れにならない（注文がある顧客は消せない）",
            all([
                references_to(self.table, &[OnDelete::Restrict, OnDelete::NoAction]),
                references_to(orders.table, &[OnDelete::Restrict, OnDelete::NoAction]),
            ]),
        )
    }
}

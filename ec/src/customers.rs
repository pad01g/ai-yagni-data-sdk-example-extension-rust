//! 顧客と、顧客を指す注文。
use ai_yagni_data_sdk::vocab::{Fact, OnDelete, SqlFile, Word, all, define, foreign_key, references_to, sql_empty};

use crate::orders::Orders;

const PII_OUTSIDE: SqlFile = SqlFile::new("pii_outside.sql", include_str!("sql/pii_outside.sql"));

/// 顧客の表。
///
/// pii は個人情報の列の名前の一部（大文字と小文字は区別しない）。`email` なら `notify_email` も `email_address` も
/// `billing_email_hash` も個人情報とみなす。略した名前（`addr`・`tel`）は、その形も並べる（`addr` は `address` も含む）。
/// 名前に表れない個人情報（`note` に書いた住所など）は見つけられない。
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
            all(self
                .pii
                .iter()
                .map(|fragment| sql_empty(PII_OUTSIDE, &[("customers", self.table), ("fragment", fragment)]))),
        )
    }

    /// 注文の顧客の列が、顧客の ID を指す外部キー。
    fn orders_point_here(&self, orders: &Orders) -> Fact {
        foreign_key(orders.table, &[orders.customer], self.table, &[self.id])
    }

    pub fn are_referred_to_by(&self, orders: &Orders) -> Word {
        define(
            "orders_refer_to_customers",
            "注文は顧客を ID で指す（メールや氏名で指さない）",
            self.orders_point_here(orders),
        )
    }

    pub fn erasure_takes_their_orders(&self, orders: &Orders) -> Word {
        define(
            "erasing_a_customer_takes_their_orders",
            "顧客の行を消すと、その顧客の注文と明細も消える（個人と結び付く記録を残さない）",
            all([
                self.orders_point_here(orders),
                foreign_key(orders.lines.table, &[orders.lines.order], orders.table, &[orders.id]),
                references_to(self.table, &[OnDelete::Cascade]),
                references_to(orders.table, &[OnDelete::Cascade]),
            ]),
        )
    }

    pub fn orders_outlive_them(&self, orders: &Orders) -> Word {
        define(
            "orders_outlive_customers",
            "顧客の行を消しても注文は道連れにならない（注文がある顧客は消せない）",
            all([
                self.orders_point_here(orders),
                references_to(self.table, &[OnDelete::Restrict, OnDelete::NoAction]),
                references_to(orders.table, &[OnDelete::Restrict, OnDelete::NoAction]),
            ]),
        )
    }
}

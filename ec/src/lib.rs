//! 層2の例: SDK（層1）を EC の業務の形で拡張したフレームワーク。
//!
//! 構成子（[`Catalog`]・[`Orders`]・[`Customers`]・[`Payments`]）は表と列の名前を受け取り、業務の言葉（語）を作る。
//! どの語も、一段たどれば層1の言明（スキーマの事実か、違反を探す SQL ファイル）に落ちる。
//! SQL は `src/sql/*.sql` に置き、表と列は `{{名前}}` で差し込む（このソースは SQL を文字列で持たない）。
//!
//! 当てはめ（プロジェクトの表名で構成子を呼び、語に名前を付けて並べること）は、利用者のプロジェクトがする。
//! 例: backend-rust の `datafutures/src/shop.rs`。
mod catalog;
mod customers;
mod orders;
mod payments;

use ai_yagni_data_sdk::vocab::{Fact, Verdict, Word, all, define};

pub use crate::catalog::{Catalog, Variants};
pub use crate::customers::Customers;
pub use crate::orders::{Lines, Orders};
pub use crate::payments::Payments;

/// 税の端数処理（Verus で証明済み）。`order_taxes_match_rates` が DB の上で確かめる規則と同じ。
pub use ai_yagni_data_ec_verified::{BASIS, tax};

/// テストに使う EC の形（backend-rust の `schema.sql` と `fixtures/0001_shop.sql` のバイト単位の写し）。
///
/// 写しがずれたら、backend-rust のテスト（`datafutures` の `the_extension_tests_the_same_shop`）が失敗する。
pub mod shop {
    pub const SCHEMA: &str = include_str!("../tests/fixtures/shop/schema.sql");
    pub const DATA: &str = include_str!("../tests/fixtures/shop/0001_shop.sql");
}

/// 言明を前から順に確かめ、成り立たないものがあればそこで止める（その結果を返す）。
///
/// 後ろの言明が前の言明の確かめる表や列を読むとき（列があるか → その列の値を読む SQL）、前が成り立たなければ
/// 後ろは確かめられない（SQL が失敗して Error）。SDK の `all` はそれを Error にするので、ここでは確かめずに止め、
/// 前の言明が成り立たない理由（列がない）を返す。構造の言明を前に、データの言明を後ろに並べる。
///
/// 材料（parts）は残らない。書き間違いのある言明を含むなら、検査の規則の節に出るように SDK の `all` のまま返す。
pub(crate) fn in_order(facts: Vec<Fact>) -> Fact {
    if facts.is_empty() || facts.iter().any(|f| !f.flaws().is_empty()) {
        return all(facts);
    }
    let says = facts.iter().map(Fact::says).collect::<Vec<_>>().join(" and ");
    Fact::new(says, move |db| {
        for f in &facts {
            match f.holds(db) {
                Verdict::Pass => {}
                other => return other,
            }
        }
        Verdict::Pass
    })
}

/// 語を前から順に確かめ、成り立たない語があればそこで止める語（未来の `realized_if` に、then の語をまとめて渡すため）。
///
/// 構造の語（列や表があるか）を前に、データの語を後ろに並べる（[`in_order`] と同じ）。
pub fn all_of(name: &str, meaning: &str, words: &[&Word]) -> Word {
    define(name, meaning, in_order(words.iter().map(|w| w.fact()).collect()))
}

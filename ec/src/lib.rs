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

use ai_yagni_data_sdk::vocab::{Fact, SqlFile, Word, define, seq, sql_empty};

pub use crate::catalog::{Catalog, Variants};
pub use crate::customers::Customers;
pub use crate::orders::{Lines, Orders, RecordGuards, ShipmentGuards};
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

const TRIGGER_MISSING: SqlFile = SqlFile::new("trigger_missing.sql", include_str!("sql/trigger_missing.sql"));

/// table に、event（`INSERT`・`UPDATE`・`DELETE`）の前に走る name という名前のトリガーがある。中身は見ない。
///
/// SDK の `guarded_against` は名前を見ないので、ほかの未来が同じ表に足したトリガー（中身が空でも）で成り立ってしまう。
/// 語が守りを言うときは、守るトリガーを名前で指す。何を拒むかは、未来の `rejects_with` がその名前で確かめる
/// （トリガーの SIGNAL の文の先頭に、自分の名前を書いておく）。
pub(crate) fn guarded_by(table: &str, event: &str, name: &str) -> Fact {
    sql_empty(TRIGGER_MISSING, &[("table", table), ("event", event), ("trigger", name)])
}

/// 語を前から順に確かめ、成り立たない語があればそこで止める語（未来の `realized_if` に、then の語をまとめて渡すため）。
///
/// SDK の `seq` で作る。前の語が成り立たなければ（確かめられなくても）、後ろの語は確かめずにその結果を返す。
/// 構造の語（列や表があるか）を前に、それを読むデータの語を後ろに並べる。材料の語は残る
/// （語の依存と、書き間違いの報告に出る）。
pub fn all_of(name: &str, meaning: &str, words: &[&Word]) -> Word {
    define(name, meaning, seq(words.iter().map(|w| w.fact())))
}

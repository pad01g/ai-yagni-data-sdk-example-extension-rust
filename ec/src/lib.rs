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

pub use crate::catalog::{Catalog, Variants};
pub use crate::customers::Customers;
pub use crate::orders::{Lines, Orders};
pub use crate::payments::Payments;

/// 税の端数処理（Verus で証明済み）。`order_taxes_match_rates` が DB の上で確かめる規則と同じ。
pub use ai_yagni_data_ec_verified::{BASIS, tax};

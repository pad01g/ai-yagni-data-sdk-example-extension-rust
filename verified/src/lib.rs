//! EC のフレームワークのうち、Verus で証明する部分: 税の端数処理。
//!
//! 税は明細ごとに「数量 × 単価 × 税率（basis point）÷ 10000 の切り捨て」で計算し、注文の税はその和とする。
//! フレームワークの語 `order_taxes_match_rates`（`tax_mismatch.sql`）は、DB の上で同じ規則を確かめる。
//! ここでは、この計算があふれないときは仕様どおりの値を返し、税が税抜きの額を超えないことを証明する。
// Verus の検証器が仕様を知っている形（比較の連なり）で書くので、clippy の言い換えの提案は当てない。
#![allow(clippy::manual_range_contains)]

use vstd::prelude::*;

verus! {

/// 税率の分母（1 basis point = 0.01%）。
pub const BASIS: u64 = 10_000;

pub open spec fn tax_spec(amount: int, rate_bp: int) -> int {
    (amount * rate_bp) / (BASIS as int)
}

/// 税抜きの額 amount に税率 rate_bp をかけた税（切り捨て）。積があふれるなら None。
pub fn tax(amount: u64, rate_bp: u16) -> (r: Option<u64>)
    ensures
        r matches Some(t) ==> t == tax_spec(amount as int, rate_bp as int),
        r matches Some(t) ==> (rate_bp <= BASIS ==> t <= amount),
        amount as int * rate_bp as int <= u64::MAX as int ==> r.is_some(),
{
    let rate = rate_bp as u64;
    match amount.checked_mul(rate) {
        None => None,
        Some(p) => {
            let t = p / BASIS;
            proof {
                if rate_bp <= BASIS {
                    assert(amount as int * rate_bp as int <= amount as int * (BASIS as int)) by (nonlinear_arith)
                        requires rate_bp as int <= BASIS as int, amount as int >= 0;
                    assert((amount as int * (BASIS as int)) / (BASIS as int) == amount as int) by (nonlinear_arith);
                    assert((amount as int * rate_bp as int) / (BASIS as int)
                        <= (amount as int * (BASIS as int)) / (BASIS as int)) by (nonlinear_arith)
                        requires amount as int * rate_bp as int <= amount as int * (BASIS as int);
                }
            }
            Some(t)
        }
    }
}

} // verus!

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn taxes_round_down() {
        assert_eq!(tax(3998, 800), Some(319));
        assert_eq!(tax(1000, 1000), Some(100));
        assert_eq!(tax(1, 10_000), Some(1));
        assert_eq!(tax(u64::MAX, 2), None);
    }
}

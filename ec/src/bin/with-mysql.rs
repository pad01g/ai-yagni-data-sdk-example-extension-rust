//! テスト用: MySQL を 1 つ立て、`AI_YAGNI_MYSQL` にそのコンテナ名を入れてコマンドを走らせる（`with-mysql cargo test`）。
fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    std::process::exit(ai_yagni_data_sdk::mysql::with_server(&args));
}

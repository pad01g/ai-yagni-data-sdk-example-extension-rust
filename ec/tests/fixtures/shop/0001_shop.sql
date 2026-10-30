-- 代表データ。本番にありそうな形を小さく持つ: 日本語の氏名と住所、在庫 0 の商品、発送済みと支払済みの注文、
-- 決済が拒否された注文（決済の参照を持たない）、決済が決着していない注文（在庫は引き当て済み）、取り消した注文、
-- 同じ商品の価格が注文の後に変わったもの（明細の単価は注文した時点の写し）。
-- 注文の合計は明細の畳み込みと一致させる（今の不変条件）。
-- 2 人ともパスワードは "password"（argon2id）。手元の compose でもこのデータを入れる。

INSERT INTO customers (id, email, password_hash, full_name, address, created_at) VALUES
  (1, 'hanako@example.com', '$argon2id$v=19$m=19456,t=2,p=1$PbhbkLnrUMnhKKg4a8PJQA$OKejesFcn3ysRMQpWTdswoa1FwmNYbXLpbwbaHCuK8A', '山田 花子', '東京都千代田区千代田 1-1', '2026-01-05 09:00:00.000'),
  (2, 'bob@example.com',    '$argon2id$v=19$m=19456,t=2,p=1$PbhbkLnrUMnhKKg4a8PJQA$OKejesFcn3ysRMQpWTdswoa1FwmNYbXLpbwbaHCuK8A', 'Bob Smith', '1 Main St, Springfield',  '2026-02-10 12:30:00.000');

INSERT INTO products (id, code, name, description, price, stock) VALUES
  (1, 'TSHIRT-001', 'T シャツ',     '綿 100% の白い T シャツ。',       2500, 20),
  (2, 'MUG-001',    'マグカップ',   '350ml の陶器のマグカップ。',       1200,  8),
  (3, 'NOTE-001',   'ノート',       'A5 の方眼ノート。80 枚。',          480, 50),
  (4, 'STICKER-01', 'ステッカー',   '耐水性のロゴステッカー。',          300,  0),
  (5, 'TOTE-001',   'トートバッグ', '帆布のトートバッグ。',             1800,  2);

-- 花子さんの注文（発送済み）。T シャツは注文の後に 2,200 円から 2,500 円に値上げした。
-- 5 は決済サービスに届かないまま残った注文（トートバッグ 1 つを引き当て済み。proxy の突き合わせが片付ける）。
INSERT INTO orders (id, customer_id, request_key, total, payment_key, payment_ref, status, created_at) VALUES
  (1, 1, '6f1c2a0e-0b6f-4c55-9d43-0d8a3c1e0001', 5600, NULL, 'pay_0001', 'shipped',   '2026-03-01 10:00:00.000'),
  (2, 2, '6f1c2a0e-0b6f-4c55-9d43-0d8a3c1e0002', 1680, NULL, 'pay_0002', 'paid',      '2026-03-02 15:20:00.000'),
  (3, 2, '6f1c2a0e-0b6f-4c55-9d43-0d8a3c1e0003', 1800, NULL, NULL,       'declined',  '2026-03-03 08:45:00.000'),
  (4, 1, '6f1c2a0e-0b6f-4c55-9d43-0d8a3c1e0004',  300, NULL, 'pay_0004', 'cancelled', '2026-03-04 11:00:00.000'),
  (5, 1, '6f1c2a0e-0b6f-4c55-9d43-0d8a3c1e0005', 1800, '0b0e7a52-6a8f-4d3e-9c1b-5f2a7d9e0005', NULL, 'pending', '2026-03-05 09:30:00.000');

INSERT INTO order_items (order_id, line_no, product_id, quantity, unit_price) VALUES
  (1, 1, 1, 2, 2200),
  (1, 2, 2, 1, 1200),
  (2, 1, 3, 1,  480),
  (2, 2, 2, 1, 1200),
  (3, 1, 5, 1, 1800),
  (4, 1, 4, 1,  300),
  (5, 1, 5, 1, 1800);

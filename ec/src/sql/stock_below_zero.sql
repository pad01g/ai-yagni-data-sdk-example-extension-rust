-- 在庫が 0 を下回る商品（在庫が NULL のものも数える）。
SELECT p.{{product_id}} AS id, p.{{stock}} AS stock
FROM {{products}} p
WHERE p.{{stock}} < 0 OR p.{{stock}} IS NULL

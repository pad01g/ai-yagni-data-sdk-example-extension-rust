-- 税率が 0〜10000 basis point（0〜100%）の外にある明細（NULL も数える）。
SELECT i.{{line_order}} AS order_id, i.{{tax_rate}} AS tax_rate
FROM {{lines}} i
WHERE i.{{tax_rate}} < 0 OR i.{{tax_rate}} > 10000 OR i.{{tax_rate}} IS NULL

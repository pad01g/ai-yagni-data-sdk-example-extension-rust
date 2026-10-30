-- 明細が写した通貨が、注文の通貨と違う明細（どちらかが NULL のものも数える）。
-- 商品の今の通貨とは比べない（明細の通貨は、単価と同じく注文した時点の写し）。
SELECT i.{{line_order}} AS order_id, i.{{line_currency}} AS line_currency, o.{{order_currency}} AS order_currency
FROM {{lines}} i
JOIN {{orders}} o ON o.{{order_id}} = i.{{line_order}}
WHERE NOT (o.{{order_currency}} <=> i.{{line_currency}})
   OR o.{{order_currency}} IS NULL OR i.{{line_currency}} IS NULL

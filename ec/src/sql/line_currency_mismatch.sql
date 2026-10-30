-- 商品の通貨が、注文の通貨と違う明細（どちらかが NULL のものも数える）。
SELECT i.{{line_order}} AS order_id, i.{{line_product}} AS product_id,
       o.{{order_currency}} AS order_currency, p.{{product_currency}} AS product_currency
FROM {{lines}} i
JOIN {{orders}} o ON o.{{order_id}} = i.{{line_order}}
JOIN {{products}} p ON p.{{product_id}} = i.{{line_product}}
WHERE NOT (o.{{order_currency}} <=> p.{{product_currency}})
   OR o.{{order_currency}} IS NULL OR p.{{product_currency}} IS NULL

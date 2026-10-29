-- 税が、明細ごとの「数量 × 単価 × 税率 ÷ 10000 の切り捨て」の和と一致しない注文（verified の tax と同じ規則）。
SELECT o.{{order_id}} AS id, o.{{tax}} AS tax,
       COALESCE(SUM(FLOOR(i.{{quantity}} * i.{{unit_price}} * i.{{tax_rate}} / 10000)), 0) AS expected
FROM {{orders}} o
LEFT JOIN {{lines}} i ON i.{{line_order}} = o.{{order_id}}
GROUP BY o.{{order_id}}, o.{{tax}}
HAVING o.{{tax}} <> expected

-- 税が、明細ごとの「数量 × 単価 × 税率 ÷ 10000 の切り捨て」の和と一致しない注文（verified の tax と同じ規則）。
-- 注文の額をまとめてから切り捨てるのではなく、明細ごとに切り捨ててから足す。
-- 税・数量・単価・税率のどれかが NULL なら、一致しないものとして数える。
SELECT o.{{order_id}} AS id, o.{{tax}} AS tax,
       COALESCE(SUM((i.{{quantity}} * i.{{unit_price}} * i.{{tax_rate}}) DIV 10000), 0) AS expected
FROM {{orders}} o
LEFT JOIN {{lines}} i ON i.{{line_order}} = o.{{order_id}}
GROUP BY o.{{order_id}}, o.{{tax}}
HAVING NOT (o.{{tax}} <=> expected)
    OR SUM(i.{{line_order}} IS NOT NULL
           AND (i.{{quantity}} IS NULL OR i.{{unit_price}} IS NULL OR i.{{tax_rate}} IS NULL)) > 0

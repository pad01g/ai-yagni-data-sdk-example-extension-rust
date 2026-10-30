-- 合計が明細（数量 × 単価）の畳み込みと一致しない注文。
-- 合計・数量・単価のどれかが NULL なら、一致しないものとして数える。
SELECT o.{{order_id}} AS id, o.{{total}} AS total,
       COALESCE(SUM(i.{{quantity}} * i.{{unit_price}}), 0) AS expected
FROM {{orders}} o
LEFT JOIN {{lines}} i ON i.{{line_order}} = o.{{order_id}}
GROUP BY o.{{order_id}}, o.{{total}}
HAVING NOT (o.{{total}} <=> expected)
    OR SUM(i.{{line_order}} IS NOT NULL AND (i.{{quantity}} IS NULL OR i.{{unit_price}} IS NULL)) > 0

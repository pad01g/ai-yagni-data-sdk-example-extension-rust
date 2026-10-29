-- 合計が明細（数量 × 単価）の畳み込みと一致しない注文。
SELECT o.{{order_id}} AS id, o.{{total}} AS total,
       COALESCE(SUM(i.{{quantity}} * i.{{unit_price}}), 0) AS expected
FROM {{orders}} o
LEFT JOIN {{lines}} i ON i.{{line_order}} = o.{{order_id}}
GROUP BY o.{{order_id}}, o.{{total}}
HAVING o.{{total}} <> expected

-- 発送済み（status = 'shipped'）なのに、発送した数が注文した数に届かない明細。
-- 状態の値 'shipped' はこのフレームワークの約束。
SELECT i.{{line_order}} AS order_id, i.{{quantity}} AS quantity, i.{{shipped}} AS shipped
FROM {{lines}} i
JOIN {{orders}} o ON o.{{order_id}} = i.{{line_order}}
WHERE o.{{status}} = 'shipped' AND i.{{shipped}} <> i.{{quantity}}

-- 発送済みの注文なのに、発送した数が注文した数と一致しない明細（どちらかが NULL のものも数える）。
-- 発送済みを表す状態の値は {{'shipped_status'}} で差し込む。
SELECT i.{{line_order}} AS order_id, i.{{quantity}} AS quantity, i.{{shipped}} AS shipped
FROM {{lines}} i
JOIN {{orders}} o ON o.{{order_id}} = i.{{line_order}}
WHERE o.{{status}} = {{'shipped_status'}}
  AND (NOT (i.{{shipped}} <=> i.{{quantity}}) OR i.{{quantity}} IS NULL)

-- 指すバリエーションが、明細の商品のものでない明細（バリエーションを指さない明細は数えない）。
SELECT i.{{line_order}} AS order_id, i.{{line_product}} AS product_id, i.{{variant}} AS variant_id
FROM {{lines}} i
LEFT JOIN {{variants}} v ON v.{{variant_id}} = i.{{variant}}
WHERE i.{{variant}} IS NOT NULL AND NOT (v.{{variant_product}} <=> i.{{line_product}})

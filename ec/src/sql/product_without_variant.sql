-- バリエーションを 1 つも持たない商品（どの商品も、少なくとも 1 つのバリエーションとして売る）。
SELECT p.{{product_id}} AS id
FROM {{products}} p
WHERE NOT EXISTS (SELECT 1 FROM {{variants}} v WHERE v.{{variant_product}} = p.{{product_id}})

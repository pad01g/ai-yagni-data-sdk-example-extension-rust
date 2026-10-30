-- 名前が {{'check'}} の CHECK 制約のうち、{{'reference'}} が NULL でないことを条件に含まないもの。
-- MySQL が整えた条件の文（`payment_ref` is not null）から逆引用符を除いて探す。
SELECT cc.CONSTRAINT_NAME AS name, cc.CHECK_CLAUSE AS clause
FROM information_schema.CHECK_CONSTRAINTS cc
JOIN information_schema.TABLE_CONSTRAINTS tc
  ON tc.CONSTRAINT_SCHEMA = cc.CONSTRAINT_SCHEMA AND tc.CONSTRAINT_NAME = cc.CONSTRAINT_NAME
WHERE cc.CONSTRAINT_SCHEMA = DATABASE()
  AND tc.TABLE_NAME = {{'table'}}
  AND cc.CONSTRAINT_NAME = {{'check'}}
  AND LOCATE(CONCAT(LOWER({{'reference'}}), ' is not null'),
             REPLACE(LOWER(cc.CHECK_CLAUSE), CHAR(96 USING utf8mb4), '')) = 0

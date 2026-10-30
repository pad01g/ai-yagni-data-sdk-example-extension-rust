-- 名前が {{'check'}} の CHECK 制約のうち、{{'reference'}} が NULL でないことを条件に含まないもの。
-- MySQL が整えた条件の文（(`payment_ref` is not null)）を、逆引用符ごと探す。逆引用符で区切るので、
-- 名前の一部が同じ別の列（`x_payment_ref` is not null）は数えない。
-- 条件の意味（どの状態で参照を持つか）までは見ない。それは今の probe で確かめる。
SELECT cc.CONSTRAINT_NAME AS name, cc.CHECK_CLAUSE AS clause
FROM information_schema.CHECK_CONSTRAINTS cc
JOIN information_schema.TABLE_CONSTRAINTS tc
  ON tc.CONSTRAINT_SCHEMA = cc.CONSTRAINT_SCHEMA AND tc.CONSTRAINT_NAME = cc.CONSTRAINT_NAME
WHERE cc.CONSTRAINT_SCHEMA = DATABASE()
  AND tc.TABLE_NAME = {{'table'}}
  AND cc.CONSTRAINT_NAME = {{'check'}}
  AND LOCATE(CONCAT(CHAR(96 USING utf8mb4), LOWER({{'reference'}}), CHAR(96 USING utf8mb4), ' is not null'),
             LOWER(cc.CHECK_CLAUSE)) = 0

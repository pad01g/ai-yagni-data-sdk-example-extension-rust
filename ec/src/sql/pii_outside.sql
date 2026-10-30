-- 名前に {{'fragment'}} を含む列のうち、{{'customers'}} の表の外にあるもの（大文字と小文字は区別しない）。
-- 名前の一部で探すので、email_address や billing_email_hash も見つかる。
SELECT c.TABLE_NAME AS t, c.COLUMN_NAME AS c
FROM information_schema.COLUMNS c
WHERE c.TABLE_SCHEMA = DATABASE()
  AND LOWER(c.TABLE_NAME) <> LOWER({{'customers'}})
  AND LOCATE(LOWER({{'fragment'}}), LOWER(c.COLUMN_NAME)) > 0
ORDER BY c.TABLE_NAME, c.COLUMN_NAME

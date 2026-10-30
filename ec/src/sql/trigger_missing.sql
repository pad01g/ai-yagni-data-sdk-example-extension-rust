-- {{'table'}} に、{{'event'}} の前に走る {{'trigger'}} という名前のトリガーがなければ 1 行を返す（中身は見ない）。
SELECT {{'table'}} AS t, {{'event'}} AS event, {{'trigger'}} AS missing_trigger
FROM DUAL
WHERE NOT EXISTS (
  SELECT 1 FROM information_schema.TRIGGERS tr
  WHERE tr.TRIGGER_SCHEMA = DATABASE()
    AND tr.TRIGGER_NAME = {{'trigger'}}
    AND tr.EVENT_OBJECT_TABLE = {{'table'}}
    AND tr.EVENT_MANIPULATION = {{'event'}}
    AND tr.ACTION_TIMING = 'BEFORE')

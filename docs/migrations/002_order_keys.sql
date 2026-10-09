-- Migration 2: task order keys move to the `fractional-indexing` format (keys like a0, a1, b00),
-- whose length grows very slowly however many tasks are added at the top or the bottom. The old
-- keys grew a character every few inserts. Every task gets a fresh key in its current order:
-- 'a' + 1 digit for the first 62, 'b' + 2 digits up to 3,844, 'c' + 3 digits up to 238,328.
WITH ranked AS (
  SELECT id, ROW_NUMBER() OVER (ORDER BY order_key, id) - 1 AS n FROM tasks
),
digits(d) AS (
  SELECT '0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz'
)
UPDATE tasks SET order_key = (
  SELECT CASE
    WHEN n < 62 THEN 'a' || substr(d, n + 1, 1)
    WHEN n < 3844 THEN 'b' || substr(d, n / 62 + 1, 1) || substr(d, n % 62 + 1, 1)
    ELSE 'c' || substr(d, n / 3844 + 1, 1) || substr(d, (n / 62) % 62 + 1, 1) || substr(d, n % 62 + 1, 1)
  END
  FROM ranked, digits
  WHERE ranked.id = tasks.id
);

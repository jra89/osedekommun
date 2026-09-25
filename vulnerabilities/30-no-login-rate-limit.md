# 30 — No login rate limiting / lockout

- **OWASP Top 10 (2021):** A07:2021 – Identification and Authentication Failures
- **Severity:** Medium
- **Difficulty:** Easy

## Where

`web/login.php:10-28` — failed login handling:

```php
if ($row) {
    ...
}
log_failed_login($u);
$err = t('login_failed');
```

`web/includes/visitlog.php:24-29` only writes the username to a visit log.
There is no attempt counter, no lockout, no delay, and no CAPTCHA.

## How it works (root cause)

The login endpoint accepts unlimited failed attempts. The only side effect is
a log line, which an attacker can ignore. Combined with weak seeded passwords
(32), this allows online brute force.

## Exploitation steps

1. Send repeated failed login attempts:

   ```bash
   for i in 1 2 3 4 5; do
     curl -s -o /dev/null -w '%{http_code} ' \
       --data 'username=nonexistent&password=wrong' \
       http://127.0.0.1:8080/login.php
   done
   ```

2. Observe that every attempt is accepted and processed normally.

## Working PoC (verified)

Five failed logins return:

```text
200 200 200 200 200
```

No account is locked, no delay is introduced, and no additional protection is
triggered.

## Expected result / verification

- Unlimited failed login attempts are possible.
- The endpoint keeps returning the normal login form after failures.

## Attack chain

```
no rate limit (this finding)
  → online brute force of weak passwords (32)
  → valid login → panel access
  → upload RCE (16) or API token theft (18/28)
```

## Notes

- The failed username is written to a world-readable log (26), which also
  enables user enumeration.
- A production login form needs throttling, lockout, and alerting.

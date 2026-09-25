# 37 — Missing security event logging

- **OWASP Top 10 (2021):** A09:2021 – Security Logging and Monitoring Failures
- **Severity:** Medium
- **Difficulty:** Easy

## Where

`conf/nginx.conf:11` disables request logging:

```nginx
access_log off;
```

`conf/php.ini:7` disables PHP error logging:

```ini
log_errors = Off
```

The high-value auth failures are not logged by the application:

- `web/api/v1/uplist/index.php:8-22` returns `401 unauthorized` for missing,
  malformed, or non-admin API tokens without writing a security event.
- `web/panel/upload.php:15-18` rejects a wrong admin re-authentication
  password with only an in-page error:

  ```php
  if (md5($pw) != $me['password_md5']) {
      $err = t('upload_badpass');
  }
  ```

The only nearby logging is `log_visit()` in `web/includes/visitlog.php`, which
records page visits and failed logins in `web/logs/visits-*.log`. It does not
record API authentication failures, upload re-auth failures, SSRF fetches, or
other security-relevant events.

## How it works (root cause)

The app has no centralized security audit trail. nginx request logs are turned
off, PHP error logs are turned off, and the application only writes a small
visit log. Failed API token use and failed privileged upload re-authentication
are therefore invisible unless an attacker also triggers a generic page visit
line.

This is not primarily about log file permissions (that is 26). It is about the
absence of useful security events.

## Exploitation steps

1. Trigger a failed API authentication event:

   ```bash
   curl -s -w '\nHTTP_CODE=%{http_code}\n' \
     -H 'Authorization: Bearer invalid-token' \
     --data-urlencode 'path=' \
     http://127.0.0.1:8080/api/v1/uplist/index.php
   ```

2. Trigger a failed admin upload re-authentication:

   ```bash
   curl -s -c /tmp/admin-37.jar -b /tmp/admin-37.jar \
     --data 'username=admin&password=Aragorn2025!' \
     http://127.0.0.1:8080/login.php

   curl -s -b /tmp/admin-37.jar \
     --data 'upload_password=wrong' \
     http://127.0.0.1:8080/panel/upload.php \
     | grep -o '<div class="error">[^<]*</div>'
   ```

3. Search the logs for a security event:

   ```bash
   grep -Rn 'upload_badpass\|Fel administratörs\|unauthorized\|invalid token\|API' \
     data/logs web/logs
   ```

## Working PoC (verified)

Failed API token:

```text
unauthorized
HTTP_CODE=401
```

Failed upload re-authentication:

```html
<div class="error">Fel administratörs lösenord.</div>
```

After both events, the log search returns:

```text
no-matching-security-logs
```

The visit log may show a generic visit or failed login, but it does not show
the API `401` or the upload re-authentication failure.

## Expected result / verification

- No nginx access log exists or is updated because `access_log off;` is set.
- No PHP error log is written because `log_errors = Off`.
- The API auth failure produces no searchable security event.
- The upload bad-password event produces no searchable security event.
- An attacker can test forged tokens and privileged upload paths without
  leaving a useful audit trail.

## Attack chain

```
forged API token / command injection (17/18/28)
  or
upload re-auth attempts / upload RCE (16/36)
  → no security event written (this finding)
  → attacker can iterate without alerting the teacher / defender
```

## Notes

- `26` covers world-readable logs and sensitive data written to logs.
- `37` covers the opposite failure: the important security events are never
  written at all.
- In a real deployment, failed authentication, privileged action attempts,
  upload attempts, and SSRF fetches should be written to a protected,
  centralized audit log.

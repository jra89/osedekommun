# 29 — Plain HTTP transport for all credentials and sessions

- **OWASP Top 10 (2021):** A02:2021 – Cryptographic Failures
- **Severity:** Medium
- **Difficulty:** Easy

## Where

`conf/nginx.conf:15` listens on plain HTTP only:

```nginx
listen 127.0.0.1:8080;
```

There is no TLS configuration anywhere in `conf/nginx.conf`.

`web/includes/config.php:4` also uses HTTP as the base URL:

```php
define('BASE_URL', 'http://localhost:8080');
```

## How it works (root cause)

All traffic — login forms, session cookies, API tokens, uploaded files, and
admin actions — travels over unencrypted HTTP. On any shared network, an
observer can capture credentials and session material in cleartext.

## Exploitation steps

1. Confirm the service is HTTP-only:

   ```bash
   curl -sI http://127.0.0.1:8080/
   ```

2. Observe that the login POST sends the password in the request body:

   ```bash
   curl -v -d 'username=ulla&password=Sommar2026' \
     http://127.0.0.1:8080/login.php 2>&1 | grep -A1 'Password'
   ```

3. On a network path, capture the request or simply read the session cookie
   from the response header.

## Working PoC (verified)

```bash
curl -sI http://127.0.0.1:8080/
```

```http
HTTP/1.1 200 OK
Server: nginx/1.28.3
X-Powered-By: PHP/8.3.17
Set-Cookie: OSEDESESSID=...; path=/
```

`grep -n 'ssl' conf/nginx.conf` returns no TLS configuration.

## Expected result / verification

- The site is reachable at `http://127.0.0.1:8080`.
- No `https` listener or TLS block exists in the nginx configuration.
- Login credentials and `OSEDESESSID` are transmitted unencrypted.

## Attack chain

```
HTTP-only transport
  → passive network sniffing captures password + session cookie
  → direct login or session replay
  → full account takeover
```

## Notes

- This amplifies 23: even if `Secure` were set, the absence of TLS means the
  whole session is exposed.
- In the lab the service is loopback-only, but the configuration itself is the
  finding.

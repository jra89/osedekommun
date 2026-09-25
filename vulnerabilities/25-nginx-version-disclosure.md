# 25 — Nginx version disclosure

- **OWASP Top 10 (2021):** A05:2021 – Security Misconfiguration
- **Severity:** Low
- **Difficulty:** Easy

## Where

`conf/nginx.conf:15` only defines the listener and contains no
`server_tokens off` directive:

```nginx
listen 127.0.0.1:8080;
```

The bundled runtime is `nginx/1.28.3`.

## How it works (root cause)

Nginx is left with default header behaviour, so it reports its exact version.
The PHP side also reports its version via `X-Powered-By`. This is low-impact
reconnaissance, but it removes the last bit of obscurity and makes component
vulnerability matching easier (33, 34, 35).

## Exploitation steps

1. Request any page and inspect response headers:

   ```bash
   curl -sI http://127.0.0.1:8080/
   ```

2. Note the version strings in `Server` and `X-Powered-By`.

## Working PoC (verified)

```http
HTTP/1.1 200 OK
Server: nginx/1.28.3
X-Powered-By: PHP/8.3.17
```

A 404 response also exposes the versions:

```http
HTTP/1.1 404 Not Found
Server: nginx/1.28.3
X-Powered-By: PHP/8.3.17
```

## Expected result / verification

- `Server` contains the exact nginx version.
- `X-Powered-By` contains the exact PHP version.

## Attack chain

```
response headers reveal exact component versions
  → attacker maps known CVEs / hardening gaps
  → supports component findings 33/34/35
```

## Notes

- Set `server_tokens off;` and hide `X-Powered-By` in a real deployment.
- This finding is Low by itself; its value is as the first step in the
  component audit chain.

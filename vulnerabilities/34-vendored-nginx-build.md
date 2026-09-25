# 34 — Vendored, fixed nginx build

- **OWASP Top 10 (2021):** A06:2021 – Vulnerable and Outdated Components
- **Severity:** Low
- **Difficulty:** Easy

## Where

The exercise ships its own nginx binary under `runtime/nginx/`:

```bash
runtime/nginx/sbin/nginx -V
```

```text
nginx version: nginx/1.28.3
built by gcc 13.3.0 (Ubuntu 13.3.0-6ubuntu2~24.04.1)
built with OpenSSL 3.0.13 30 Jan 2024
TLS SNI support enabled
configure arguments: --prefix=.../runtime/nginx --sbin-path=sbin/nginx --conf-path=conf/nginx.conf --with-http_ssl_module
```

## How it works (root cause)

nginx is a vendored, fixed build with no package-manager update pipeline.
The exact version is also disclosed in response headers (25). If a CVE is
published, the only remediation path is to rebuild/replace the vendored
binary.

## Exploitation steps

1. Inspect the vendored binary:

   ```bash
   runtime/nginx/sbin/nginx -V
   ```

2. Confirm the version is exposed to clients:

   ```bash
   curl -sI http://127.0.0.1:8080/ | grep Server
   ```

## Working PoC (verified)

```text
nginx version: nginx/1.28.3
Server: nginx/1.28.3
```

## Expected result / verification

- The web server is a self-contained build.
- Its version is visible both locally and over HTTP.

## Attack chain

```
fixed vendored nginx
  → version disclosure (25)
  → attacker maps known CVEs / hardening gaps
```

## Notes

- Low impact by itself; the risk is operational (no easy patch path).
- The same pattern applies to PHP (35).

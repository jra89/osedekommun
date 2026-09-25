# OseDekommun — vulnerability overview

Teacher overview for the **OseDekommun** web-security exercise. Use this file together with the running app. Exact payloads, curls, and exploitation steps are in the numbered files in this same folder; this file is for running and evaluating a session.

## Exercise model

- A Swedish municipality-style PHP/MySQL web app, intentionally full of OWASP Top 10 (2021) problems.
- The web root is `web/`; the answer documentation lives outside the web root in `vulnerabilities/`.
- The app is local-only: no domain required, all services run from the project folder.
- `./osede run --adminvisit` also starts an admin bot that visits relevant admin pages about once per minute. This makes stored XSS payloads able to steal the admin session.
- `./osede reset` returns the app to the default seeded state.

## Starting the app

From the project root:

```bash
./osede reset
./osede run --adminvisit
./osede status
./osede stop
```

After startup the app is available at:

```text
http://127.0.0.1:8080/
```

MySQL listens locally on port `3307`.

Seeded accounts:

| Role | Username | Password | Notes |
|---|---|---|---|
| staff | `ulla` | `Sommar2026` | main student account; can write news, messages, notes, and see logs |
| admin | `admin` | `Aragorn2025!` | admin account; upload and uplist functionality; password hint reachable through notes IDOR |
| staff | `bengt` | `Hosten2024` | additional seeded staff account |
| staff | `karin` | not intended for direct guessing | seeded account; DB hash can be inspected for documentation/cracking discussions |

## Where everything is

| Path | Purpose |
|---|---|
| `web/` | web application source, web root |
| `sql/` | schema and seed data |
| `conf/` | nginx and PHP-FPM configuration |
| `runtime/` | vendored nginx, PHP, and MySQL runtimes |
| `data/logs/` | runtime logs written by the controller/app |
| `web/logs/` | app visit logs, including failed login attempts |
| `poc/` | proof-of-concept scripts and UDF source |
| `tools/` | adminvisit helper used by the Rust controller |
| `control/` | Rust source for the `osede` controller |
| `vulnerabilities/` | teacher answer documentation, one numbered file per vulnerability |
| `tasks/` | project task specifications |

## Main attack chains

Suggested student journey, roughly in increasing difficulty:

1. **Orientation and recon**
   - Browse the front page and news.
   - Inspect login `Set-Cookie`, HTTP headers, and version disclosure.
   - Try the seeded weak passwords.
   - Discover XSS, IDOR, path traversal, open redirect, and plain HTTP.

2. **Broken access control**
   - Use the notes IDOR to read admin notes without proper authorization.
   - Note 159 contains the admin password hint.
   - Use image path traversal to read local files.
   - Use contact-form LFI to include local files.

3. **Injection and data access**
   - Test login, news detail, reset, message, and panel forms for SQL injection.
   - Use UNION and error-based SQLi to read data.
   - Use blind SQLi to extract hashes or other data.

4. **Stored XSS and session theft**
   - Put stored XSS in news or message subject.
   - Wait for `--adminvisit` to execute the payload.
   - Exfiltrate the admin cookie to ulla's inbox.
   - Use the insecure cookie to act as admin where possible.

5. **Privilege escalation and RCE**
   - Upload a PHP file disguised as an image after obtaining the admin password.
   - Forge or abuse the API token and use ROT13 + command injection in the upload-list API.
   - Combine failed-login log poisoning with contact-form LFI.
   - Use the old MySQL UDF chain for database-process RCE.

6. **Harder / advanced paths**
   - Error-based reset SQLi.
   - Blind news-detail SQLi data extraction.
   - Full log-poisoning RCE.
   - MySQL UDF RCE.

## OWASP Top 10 (2021) mapping

| Category | Vulnerabilities |
|---|---|
| A01:2021 – Broken Access Control | 08, 10, 11, 18, 21 |
| A02:2021 – Cryptographic Failures | 22, 27, 28, 29 |
| A03:2021 – Injection | 01, 02, 03, 04, 05, 06, 07, 12, 15, 16, 17, 38 |
| A04:2021 – Insecure Design | 09, 20 |
| A05:2021 – Security Misconfiguration | 19, 24, 25, 26 |
| A06:2021 – Vulnerable and Outdated Components | 33, 34, 35 |
| A07:2021 – Identification and Authentication Failures | 23, 30, 31, 32 |
| A08:2021 – Software and Data Integrity Failures | 36 |
| A09:2021 – Security Logging and Monitoring Failures | 37 |
| A10:2021 – SSRF | 13, 14 |

Every category has at least one instance.

## Teacher checklist

Tick each row when the student has found and demonstrated the vulnerability. Use the notes column for what the student found, missed, or discussed.

| [ ] | # | Name | Where | Category | Difficulty | Student notes |
|---|---:|---|---|---|---|---|
| [ ] | 01 | Stored XSS via news title/body | `web/news.php:27,29` — news detail title/body rendering | A03 | Easy |  |
| [ ] | 02 | Blind SQL injection on news detail | `web/news.php:5,18` — `GET /news.php?id=<SQL>` | A03 | Medium |  |
| [ ] | 03 | SQL injection in login form | `web/login.php:14` — `POST /login.php` | A03 | Easy |  |
| [ ] | 04 | UNION SQLi in password reset step 1 | `web/reset.php:16` — `POST /reset.php`, `step=1` | A03 | Medium |  |
| [ ] | 05 | Error-based SQLi in password reset step 2 | `web/reset.php:41` — `POST /reset.php`, `step=2` | A03 | Hard |  |
| [ ] | 06 | SQL injection in message form | `web/panel/messages.php` — `POST /panel/messages.php` | A03 | Medium |  |
| [ ] | 07 | SQL injection in panel INSERTs | `web/panel/news_new.php:10` and related panel insert forms | A03 | Medium |  |
| [ ] | 08 | IDOR: unauthenticated note read | `web/panel/notes.php:5-22` — `GET /panel/notes.php?action=view&id=N` | A01 | Easy |  |
| [ ] | 09 | Predictable sequential note ids | `web/panel/notes.php:29-32` | A04 | Easy |  |
| [ ] | 10 | Path traversal in image handler | `web/image.php:2,6` — `GET /image.php?file=<path>` | A01 | Easy |  |
| [ ] | 11 | Local file inclusion via contact form selector | `web/contact.php:13,15` — `GET /contact.php?form=<name>` | A01 | Easy |  |
| [ ] | 12 | Log poisoning to PHP code execution | `web/includes/visitlog.php:24-28` + `web/contact.php:15` | A03 | Hard |  |
| [ ] | 13 | SSRF in message link preview | `web/panel/messages.php:44-45` — `POST /panel/messages.php`, field `link` | A10 | Easy |  |
| [ ] | 14 | Blind SSRF in contact form | `web/contact.php:7-9` — `POST /contact.php`, field `webpage` | A10 | Easy |  |
| [ ] | 15 | Stored XSS via message subject | `web/panel/messages.php:18` — message read view | A03 | Easy |  |
| [ ] | 16 | RCE via admin image upload | `web/panel/upload.php:22-34` — `POST /panel/upload.php` | A03 | Medium |  |
| [ ] | 17 | Command injection in uplist API | `web/api/v1/uplist/index.php:24,26` — `POST /api/v1/uplist/` | A03 | Medium |  |
| [ ] | 18 | Forged API token | `web/includes/auth.php:46-54` — `check_api_token()` | A01 | Medium |  |
| [ ] | 19 | RCE via MySQL UDF | `osede_app` has `ALL PRIVILEGES` + `FILE`; PoC in `poc/udf-rce.sh` | A05 | Hard |  |
| [ ] | 20 | Weak, guessable password-reset code | `web/reset.php:23` | A04 | Easy |  |
| [ ] | 21 | Open redirect in login page | `web/login.php:7-8,23-24` — `redirectUrl` | A01 | Easy |  |
| [ ] | 22 | Unsalted MD5 password storage | `web/login.php:14`, reset/upload password checks, `sql/seed.sql` | A02 | Easy |  |
| [ ] | 23 | Insecure session cookie | `web/includes/auth.php:4-9`, `conf/php.ini:21-23` | A07 | Easy |  |
| [ ] | 24 | Verbose PHP errors | `conf/php.ini:4-6`, e.g. `web/image.php:6` | A05 | Easy |  |
| [ ] | 25 | Nginx version disclosure | `conf/nginx.conf:15`, response `Server` header | A05 | Easy |  |
| [ ] | 26 | World-readable application logs | `data/logs/`, `web/logs/` | A05 | Easy |  |
| [ ] | 27 | ROT13 used as "encryption" | `web/includes/auth.php:56-58`, `web/api/v1/uplist/index.php:24` | A02 | Easy |  |
| [ ] | 28 | Base64 API token with unverified signature | `web/includes/auth.php:40-54` | A02 | Medium |  |
| [ ] | 29 | Plain HTTP transport for credentials and sessions | `conf/nginx.conf:15`, `web/includes/config.php` | A02 | Easy |  |
| [ ] | 30 | No login rate limiting / lockout | `web/login.php:10-28` | A07 | Easy |  |
| [ ] | 31 | No reset-code rate limiting | `web/reset.php:37-53` | A07 | Medium |  |
| [ ] | 32 | Weak, guessable seeded passwords | `sql/seed.sql:4-7` | A07 | Medium |  |
| [ ] | 33 | Outdated MySQL 5.5 | bundled MySQL runtime `5.5.62` | A06 | Medium |  |
| [ ] | 34 | Vendored, fixed nginx build | `runtime/nginx/` | A06 | Easy |  |
| [ ] | 35 | Vendored, fixed PHP build | `runtime/php/` | A06 | Easy |  |
| [ ] | 36 | Upload validation trusts magic bytes only | `web/panel/upload.php:22-34` | A08 | Easy |  |
| [ ] | 37 | Missing security event logging | `conf/nginx.conf:11`, `conf/php.ini:7`, API/upload auth paths | A09 | Easy |  |
| [ ] | 38 | Reflected XSS via system status banner | `web/api/v1/status/index.php:16` + `web/includes/layout.php:33` — `GET /api/v1/status/?msg=<payload>` (message in the banner's AJAX URL) | A03 | Medium |  |

## Expected discovery order

A typical student path:

1. **Very first finds**
   - 01 stored XSS in news
   - 08 notes IDOR
   - 10 image path traversal
   - 21 open redirect
   - 23 insecure session cookie
   - 25 nginx version disclosure
   - 29 plain HTTP
   - 30 no login rate limiting
   - 32 weak seeded passwords

2. **Common second-wave finds**
   - 13 message SSRF
   - 14 contact SSRF
   - 15 message-subject stored XSS
   - 20 weak reset code
   - 22 unsalted MD5
   - 24 verbose PHP errors
    - 26 world-readable logs
    - 31 no reset rate limiting
    - 36 upload magic-byte validation
    - 37 missing security event logging
    - 38 reflected XSS in the status banner (via the Network tab: message in the AJAX URL, `Accept`/`Content-Type: text/html`)

3. **Once the student starts chaining**
   - 03 login SQLi
   - 04 reset UNION SQLi
   - 06 message SQLi
   - 07 panel INSERT SQLi
   - 09 sequential note ids
   - 16 upload RCE
   - 17 uplist command injection
   - 18 forged API token
   - 27 ROT13
   - 28 base64 API token
   - 33 outdated MySQL
   - 34 vendored nginx
   - 35 vendored PHP

4. **Harder / advanced findings**
   - 02 blind SQLi data extraction
   - 05 error-based reset SQLi
   - 12 log-poisoning RCE
   - 19 MySQL UDF RCE

## Answer key pointers

Do not put payloads in this file. Use the numbered files for the actual exploitation steps.

- Per-vulnerability answers: `vulnerabilities/01-*.md` through `vulnerabilities/38-*.md`
- Session and API token code: `web/includes/auth.php`
- PHP configuration: `conf/php.ini`
- nginx configuration: `conf/nginx.conf`
- Login and open redirect: `web/login.php`
- Reset SQLi and reset code logic: `web/reset.php`
- News SQLi and XSS sink: `web/news.php`
- Message SQLi, XSS sink, and SSRF: `web/panel/messages.php`
- Notes IDOR: `web/panel/notes.php`
- Image path traversal: `web/image.php`
- Contact LFI and SSRF: `web/contact.php`
- Upload RCE and magic-byte validation: `web/panel/upload.php`
- Uplist command injection: `web/api/v1/uplist/index.php`
- Status banner reflected XSS: `web/api/v1/status/index.php` + `web/includes/layout.php`
- Visit/failed-login log poisoning: `web/includes/visitlog.php`
- Seeded users and hashes: `sql/seed.sql`
- MySQL UDF PoC: `poc/udf-rce.sh`
- Reflected XSS PoC: `poc/reflected-xss-status.sh`
- UDF C source: `poc/lib_mysqludf_sys.c`
- Admin bot behavior: `tools/adminvisit.js`
- Controller/runtime behavior: `control/src/main.rs`

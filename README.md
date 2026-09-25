# Osede Kommun - IT-security training environment

<p align="center">
  <img src="web/images/hero.svg" alt="Osede Kommun" width="720">
</p>

A **deliberately vulnerable** PHP/MySQL web application for hands-on IT-security
training. Not a real municipality website. Everything runs locally on
`127.0.0.1`.

## Quick start (Linux)

The prebuilt control binary `./osede` is included:

```sh
./osede run                          # start MySQL, PHP-FPM, nginx
./osede run --port 9000              # different web listen port (default 8080)
./osede run --ip 0.0.0.0             # make the web site reachable from the network
./osede run --adminvisit             # also run the admin headless-browser visit
./osede run --install-service        # install as a systemd service (starts on boot)
./osede status [--port N]            # show running services and ports
./osede stop                         # stop everything
./osede reset                        # stop and wipe all local state (DB, logs, uploads)
./osede fetch                        # download missing runtime binaries only
```

All `run` flags can be combined, e.g.
`sudo ./osede run --port 9000 --ip 0.0.0.0 --adminvisit --install-service`.

Open `http://localhost:8080`. The first `run` downloads any missing runtime
binaries (with a progress bar per file), initializes the MySQL data directory,
and loads `sql/schema.sql` + `sql/seed.sql`.

## Runtime binaries (auto-downloaded)

`runtime/` (MySQL, PHP-FPM, nginx, Deno, ~1 GB unpacked) is **not** in the
repo. On first `run` - or on demand via `./osede fetch` - the control binary
downloads only the components that are missing, verifies each file against the
`SHA256SUMS` manifest of the release, and extracts it into `runtime/`.

- Download base URL: the GitHub release `v1.0.0` of this repo. Override with
  the `OSEDE_RUNTIME_BASE` environment variable if you host the tarballs
  elsewhere.
- Existing components are never re-downloaded; delete e.g. `runtime/deno/` to
  force a re-fetch.

### Updating the bundled binaries (maintainers)

1. Replace the contents of `runtime/<component>/` with the new build.
2. `./scripts/package-runtime.sh` - rebuilds `dist/runtime-*.tar.gz` + `SHA256SUMS`.
3. Upload the new assets to the release (same filenames, or bump the release
   tag referenced in `control/src/fetch.rs`), then rebuild `./osede` if you
   changed the tag.

## Lab deployment (teachers)

Strip the answer key and git metadata before handing a copy to students:

```sh
./ctf.sh   # removes poc/, vulnerabilities/, .git and all .gitignore files
```

To expose the site to the lab network and have it start automatically when
the VM boots:

```sh
sudo ./osede run --port 8080 --ip 0.0.0.0 --adminvisit --install-service
```

This writes `/etc/systemd/system/osede.service` pointing at the control
binary's current location, remembers the `--port`/`--ip`/`--adminvisit`
arguments, enables the service (auto-start on boot) and starts it now.
Manage it afterwards with `systemctl stop|start|restart osede` or `./osede stop`.

Ports: web `127.0.0.1:8080` (set in `conf/nginx.conf`), MySQL `127.0.0.1:3307`
(set in `control/src/main.rs`).

## Building the control tool (Linux)

Only needed if `control/src/main.rs` was changed. Requires a Rust toolchain
(`rustc` + `cargo`):

```sh
./scripts/build-osede.sh
```

or manually:

```sh
cd control
cargo build --release
cp target/release/osede ../osede
chmod +x ../osede
```

Run it as shown in the quick start above: `./osede run`.

## Folder layout

| Path | Contents |
|------|----------|
| `osede` | Prebuilt control binary (Rust) |
| `web/` | The vulnerable PHP site (`panel/` admin area, `forms/`, `api/`, `includes/`, `images/`, `css/`) |
| `web/uploads/` | User uploads + seeded news images (wiped on reset) |
| `sql/` | `schema.sql` (creates `osede_db`) and `seed.sql` (users, news, notes) |
| `conf/` | nginx / PHP-FPM / PHP templates, rendered to `data/conf/` on first run |
| `data/` | Runtime state only (MySQL data, logs, PIDs, sessions) - created on first run |
| `runtime/` | Runtime binaries (MySQL, PHP-FPM, nginx, Deno) - **downloaded on first run**, not in the repo |
| `dist/` | Packaged tarballs produced by `scripts/package-runtime.sh` for the GitHub release |
| `control/` | Rust source for the `./osede` control binary |
| `vulnerabilities/` | **Answer key**,  one document per vulnerability + teacher overview |
| `poc/` | **Working exploit scripts** (includes a UDF RCE PoC) |
| `tools/` | `adminvisit.js`.  Deno headless-browser script used by `./osede run --adminvisit` |
| `scripts/` | `build-osede.sh` (rebuilds the control binary), `package-runtime.sh` (builds release tarballs) |
| `ctf.sh` | strips `poc/`, `vulnerabilities/`, `.git` and `.gitignore` files for student deployment |

## Notes

- MySQL 5.5 is used on purpose so certain attacks (e.g. UDF RCE) still work.
- Verbose PHP errors are intentional to help students find information leaks.
- `./osede reset` is destructive: it wipes the database, logs, sessions, and
  uploads.
- Keep `vulnerabilities/` and `poc/` away from students as it reveals all the answers.

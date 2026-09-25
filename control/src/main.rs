mod fetch;

use std::env;
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

fn root_dir() -> PathBuf {
    // runtime/ is downloaded on demand, so it is not a valid root marker.
    if let Ok(cwd) = env::current_dir() {
        if cwd.join("web").is_dir() && cwd.join("sql").is_dir() {
            return cwd;
        }
    }
    if let Ok(exe) = env::current_exe() {
        if let Some(p) = exe.parent() {
            let p = p.to_path_buf();
            if p.join("web").is_dir() && p.join("sql").is_dir() {
                return p;
            }
        }
    }
    eprintln!("cannot find project root (need web/ and sql/)");
    std::process::exit(1);
}

fn whoami() -> String {
    env::var("USER")
        .or_else(|_| env::var("LOGNAME"))
        .unwrap_or_else(|_| "root".to_string())
}

/// (user, group) to run the web tier (nginx/php-fpm) as. php-fpm refuses to
/// run as root, so when the control process is root (e.g. via the systemd
/// service) the web tier runs as "nobody" instead.
fn web_user() -> (String, String) {
    let user = whoami();
    if user != "root" {
        let group = env::var("GROUP").unwrap_or_else(|_| user.clone());
        return (user, group);
    }
    let group = Command::new("id")
        .arg("-gn")
        .arg("nobody")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .unwrap_or_default()
        .trim()
        .to_string();
    (
        "nobody".to_string(),
        if group.is_empty() {
            "nogroup".to_string()
        } else {
            group
        },
    )
}

/// Make the directories the web tier writes to writable by web_user(),
/// which only matters when that is not the current user (i.e. we are root).
fn chown_web_dirs(root: &Path) {
    let (user, group) = web_user();
    if user == whoami() {
        return;
    }
    for d in [
        "data/run",
        "data/logs",
        "data/sessions",
        "data/tmp",
        "web/uploads",
        "web/logs",
    ] {
        if let Ok(o) = Command::new("chown")
            .arg("-R")
            .arg(format!("{user}:{group}"))
            .arg(root.join(d))
            .output()
        {
            if !o.status.success() {
                eprintln!(
                    "warning: chown {} for {} failed: {}",
                    d,
                    user,
                    String::from_utf8_lossy(&o.stderr).trim()
                );
            }
        }
    }
}

struct Opts {
    port: u16,
    ip: String,
    adminvisit: bool,
    install_service: bool,
}

fn parse_opts(args: &[String]) -> Opts {
    let mut port: u16 = 8080;
    let mut ip = "127.0.0.1".to_string();
    let mut adminvisit = false;
    let mut install_service = false;
    let mut i = 0;
    while i < args.len() {
        let a = args[i].as_str();
        if a == "--adminvisit" {
            adminvisit = true;
        } else if a == "--install-service" {
            install_service = true;
        } else if a == "--port" || a.starts_with("--port=") {
            let v = a
                .strip_prefix("--port=")
                .map(|s| s.to_string())
                .or_else(|| args.get(i + 1).cloned());
            match v.and_then(|v| v.parse().ok()) {
                Some(p) => port = p,
                None => {
                    eprintln!("error: --port expects a number");
                    std::process::exit(1);
                }
            }
            if a == "--port" {
                i += 1;
            }
        } else if a == "--ip" || a.starts_with("--ip=") {
            let v = a
                .strip_prefix("--ip=")
                .map(|s| s.to_string())
                .or_else(|| args.get(i + 1).cloned());
            match v {
                Some(v) if !v.is_empty() => ip = v,
                _ => {
                    eprintln!("error: --ip expects an address");
                    std::process::exit(1);
                }
            }
            if a == "--ip" {
                i += 1;
            }
        }
        i += 1;
    }
    Opts {
        port,
        ip,
        adminvisit,
        install_service,
    }
}

fn ensure_dirs(root: &Path) {
    for d in [
        "data/run",
        "data/logs",
        "data/mysql",
        "data/sessions",
        "data/tmp",
        "data/conf",
        "web/uploads",
        "web/logs",
    ] {
        fs::create_dir_all(root.join(d)).expect("mkdir");
    }
}

fn pid_alive(pid: i32) -> bool {
    Path::new(&format!("/proc/{}", pid)).exists()
}

fn read_pid(root: &Path, name: &str) -> Option<i32> {
    let pf = root.join("data/run").join(name);
    fs::read_to_string(&pf)
        .ok()
        .and_then(|s| s.trim().parse::<i32>().ok())
        .filter(|p| *p > 0 && pid_alive(*p))
}

fn kill_pid(pid: i32) -> bool {
    if let Ok(o) = Command::new("kill").arg("-TERM").arg(pid.to_string()).output() {
        if !o.status.success() {
            eprintln!(
                "warning: cannot send TERM to pid {} ({}); is it running as another user?",
                pid,
                String::from_utf8_lossy(&o.stderr).trim()
            );
        }
    }
    let start = Instant::now();
    while start.elapsed() < Duration::from_secs(8) && pid_alive(pid) {
        thread::sleep(Duration::from_millis(200));
    }
    if pid_alive(pid) {
        if let Ok(o) = Command::new("kill").arg("-9").arg(pid.to_string()).output() {
            if !o.status.success() {
                eprintln!(
                    "warning: cannot send KILL to pid {} ({}); is it running as another user?",
                    pid,
                    String::from_utf8_lossy(&o.stderr).trim()
                );
            }
        }
        thread::sleep(Duration::from_millis(300));
    }
    !pid_alive(pid)
}

/// Pids of processes whose cmdline contains `needle` (e.g. this project's
/// mysqld binary path). Used to find processes that are running without a
/// usable pid file.
fn find_pids(needle: &str) -> Vec<i32> {
    let mut out = Vec::new();
    if let Ok(entries) = fs::read_dir("/proc") {
        for e in entries.flatten() {
            let name = e.file_name().to_string_lossy().to_string();
            let pid: i32 = match name.parse() {
                Ok(p) if p > 0 => p,
                _ => continue,
            };
            if let Ok(cmd) = fs::read_to_string(format!("/proc/{}/cmdline", pid)) {
                if cmd.split('\0').any(|a| !a.is_empty() && a.contains(needle)) {
                    out.push(pid);
                }
            }
        }
    }
    out
}

fn stop_one(root: &Path, name: &str, needle: &str) {
    let mut pids = match read_pid(root, name) {
        Some(p) => vec![p],
        None => find_pids(needle),
    };
    if pids.is_empty() && !root.join("data/run").join(name).exists() {
        return;
    }
    for pid in pids.drain(..) {
        if !kill_pid(pid) {
            eprintln!(
                "warning: {} (pid {}) is still running after stop; \
                 it may be owned by another user (try sudo ./osede stop)",
                name, pid
            );
        }
    }
    fs::remove_file(root.join("data/run").join(name)).ok();
}

fn stop_all(root: &Path) {
    let root_s = root.to_string_lossy().to_string();
    stop_one(root, "nginx.pid", &format!("{root_s}/runtime/nginx/sbin/nginx"));
    stop_one(root, "php-fpm.pid", &format!("{root_s}/runtime/php/sbin/php-fpm"));
    stop_one(root, "adminvisit.pid", &format!("{root_s}/tools/adminvisit.js"));
    stop_one(root, "mysqld.pid", &format!("{root_s}/runtime/mysql/root/bin/mysqld"));
}

fn init_mysql(root: &Path) {
    let script = root.join("runtime/mysql/root/scripts/mysql_install_db");
    let basedir = root.join("runtime/mysql/root");
    let lib = root.join("runtime/mysql/extra-libs");
    let datadir = root.join("data/mysql");
    let logf = fs::File::create(root.join("data/logs/mysql-init.log")).expect("log");
    let mut c = Command::new(&script);
    c.arg("--no-defaults");
    c.arg(format!("--basedir={}", basedir.display()));
    c.arg(format!("--datadir={}", datadir.display()));
    c.arg(format!("--user={}", whoami()));
    c.env("LD_LIBRARY_PATH", lib.to_string_lossy().to_string());
    c.stdin(Stdio::null());
    c.stdout(logf.try_clone().expect("clone"));
    c.stderr(logf);
    let st = c.status().expect("run mysql_install_db");
    if !st.success() {
        eprintln!("mysql_install_db failed, see data/logs/mysql-init.log");
        std::process::exit(1);
    }
}

fn start_mysql(root: &Path) {
    let bin = root.join("runtime/mysql/root/bin/mysqld");
    let lib = root.join("runtime/mysql/extra-libs");
    let logf = fs::File::create(root.join("data/logs/mysql-start.log")).expect("log");
    let mut c = Command::new(&bin);
    c.arg("--datadir").arg(root.join("data/mysql"));
    c.arg("--socket").arg(root.join("data/run/mysql.sock"));
    c.arg("--port=3307");
    c.arg("--bind-address=127.0.0.1");
    c.arg("--skip-name-resolve");
    c.arg("--secure-file_priv=");
    c.arg(format!(
        "--plugin-dir={}",
        root.join("runtime/mysql/root/lib/plugin").display()
    ));
    c.arg(format!("--lc-messages-dir={}", root.join("runtime/mysql/root/share").display()));
    c.arg("--pid-file").arg(root.join("data/run/mysqld.pid"));
    c.arg("--user").arg(whoami());
    c.arg(format!(
        "--log-error={}",
        root.join("data/logs/mysql-error.log").display()
    ));
    c.env("LD_LIBRARY_PATH", lib.to_string_lossy().to_string());
    c.stdin(Stdio::null());
    c.stdout(logf.try_clone().expect("clone"));
    c.stderr(logf);
    c.spawn().expect("spawn mysqld");
}

fn libpath(root: &Path) -> String {
    root.join("runtime/mysql/extra-libs").to_string_lossy().to_string()
}

fn port_in_use(port: u16) -> bool {
    std::net::TcpStream::connect(format!("127.0.0.1:{}", port)).is_ok()
}

fn print_log_tail(root: &Path, name: &str, n: usize) {
    if let Ok(s) = fs::read_to_string(root.join("data/logs").join(name)) {
        eprintln!("last lines of data/logs/{}:", name);
        for line in s.lines().rev().take(n) {
            eprintln!("  {line}");
        }
    }
}

fn wait_mysql(root: &Path) -> bool {
    let bin = root.join("runtime/mysql/root/bin/mysql");
    let sock = root.join("data/run/mysql.sock");
    let start = Instant::now();
    while start.elapsed() < Duration::from_secs(60) {
        if let Ok(o) = Command::new(&bin)
            .arg("--socket")
            .arg(&sock)
            .arg("-u")
            .arg("root")
            .arg("-e")
            .arg("SELECT 1")
            .env("LD_LIBRARY_PATH", libpath(root))
            .output()
        {
            if o.status.success() {
                return true;
            }
        }
        thread::sleep(Duration::from_millis(500));
    }
    false
}

fn setup_db(root: &Path) {
    let bin = root.join("runtime/mysql/root/bin/mysql");
    let sock = root.join("data/run/mysql.sock");
    let out = Command::new(&bin)
        .arg("--socket")
        .arg(&sock)
        .arg("-u")
        .arg("root")
        .arg("-e")
        .arg("CREATE DATABASE IF NOT EXISTS osede_db CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci;")
        .env("LD_LIBRARY_PATH", libpath(root))
        .output()
        .expect("create db");
    if !out.status.success() {
        eprintln!("create database failed: {}", String::from_utf8_lossy(&out.stderr));
        std::process::exit(1);
    }
    for f in ["sql/schema.sql", "sql/seed.sql"] {
        let p = root.join(f);
        let file = fs::File::open(&p).expect("open sql");
        let out = Command::new(&bin)
            .arg("--socket")
            .arg(&sock)
            .arg("-u")
            .arg("root")
            .arg("--default-character-set=utf8mb4")
            .arg("osede_db")
            .stdin(file)
            .env("LD_LIBRARY_PATH", libpath(root))
            .output()
            .expect("load sql");
        if !out.status.success() {
            eprintln!("loading {} failed: {}", f, String::from_utf8_lossy(&out.stderr));
            std::process::exit(1);
        }
    }
    let sql = "CREATE USER 'osede_app'@'127.0.0.1' IDENTIFIED BY 'OseDe2026app';\
CREATE USER 'osede_app'@'localhost' IDENTIFIED BY 'OseDe2026app';\
CREATE USER 'osede_read'@'127.0.0.1' IDENTIFIED BY 'OseDe2026read';\
CREATE USER 'osede_read'@'localhost' IDENTIFIED BY 'OseDe2026read';\
GRANT ALL PRIVILEGES ON *.* TO 'osede_app'@'127.0.0.1';\
GRANT ALL PRIVILEGES ON *.* TO 'osede_app'@'localhost';\
GRANT SELECT ON osede_db.news TO 'osede_read'@'127.0.0.1';\
GRANT SELECT ON osede_db.users TO 'osede_read'@'127.0.0.1';\
GRANT SELECT ON osede_db.news TO 'osede_read'@'localhost';\
GRANT SELECT ON osede_db.users TO 'osede_read'@'localhost';\
FLUSH PRIVILEGES;";
    let out = Command::new(&bin)
        .arg("--socket")
        .arg(&sock)
        .arg("-u")
        .arg("root")
        .arg("-e")
        .arg(sql)
        .env("LD_LIBRARY_PATH", libpath(root))
        .output()
        .expect("create users");
    if !out.status.success() {
        eprintln!("db user setup failed: {}", String::from_utf8_lossy(&out.stderr));
        std::process::exit(1);
    }
}

fn write_conf(root: &Path, opts: &Opts) {
    let (user, group) = web_user();
    let root_s = root.to_string_lossy().to_string();
    let nginx = root.join("runtime/nginx").to_string_lossy().to_string();
    let port = opts.port.to_string();
    for (src, dst) in [
        ("conf/php-fpm.conf", "data/conf/php-fpm.conf"),
        ("conf/nginx.conf", "data/conf/nginx.conf"),
    ] {
        let t = fs::read_to_string(root.join(src)).expect("read conf");
        let t = t
            .replace("@ROOT@", &root_s)
            .replace("@USER@", &user)
            .replace("@GROUP@", &group)
            .replace("@NGINX@", &nginx)
            .replace("@WEBIP@", &opts.ip)
            .replace("@WEBPORT@", &port);
        fs::write(root.join(dst), t).expect("write conf");
    }
    let fp = root.join("runtime/nginx/conf/fastcgi_params");
    fs::copy(&fp, root.join("data/conf/fastcgi_params")).expect("copy fastcgi_params");
}

fn start_fpm(root: &Path) {
    let bin = root.join("runtime/php/sbin/php-fpm");
    let logf = fs::File::create(root.join("data/logs/php-fpm-start.log")).expect("log");
    Command::new(&bin)
        .arg("--fpm-config")
        .arg(root.join("data/conf/php-fpm.conf"))
        .stdin(Stdio::null())
        .stdout(logf.try_clone().expect("clone"))
        .stderr(logf)
        .spawn()
        .expect("spawn php-fpm");
}

fn wait_sock(root: &Path) -> bool {
    let sock = root.join("data/run/php-fpm.sock");
    let start = Instant::now();
    while start.elapsed() < Duration::from_secs(30) {
        if sock.exists() {
            return true;
        }
        thread::sleep(Duration::from_millis(200));
    }
    false
}

fn start_nginx(root: &Path) {
    let bin = root.join("runtime/nginx/sbin/nginx");
    let logf = fs::File::create(root.join("data/logs/nginx-start.log")).expect("log");
    Command::new(&bin)
        .arg("-c")
        .arg(root.join("data/conf/nginx.conf"))
        .env(
            "LD_LIBRARY_PATH",
            root.join("runtime/nginx/extra-libs").to_string_lossy().to_string(),
        )
        .stdin(Stdio::null())
        .stdout(logf.try_clone().expect("clone"))
        .stderr(logf)
        .spawn()
        .expect("spawn nginx");
}

fn wait_http(port: u16) -> bool {
    let start = Instant::now();
    while start.elapsed() < Duration::from_secs(30) {
        if let Ok(mut s) = std::net::TcpStream::connect(format!("127.0.0.1:{}", port)) {
            let req = "GET / HTTP/1.0\r\nHost: localhost\r\nConnection: close\r\n\r\n";
            if s.write_all(req.as_bytes()).is_ok() {
                let mut buf = [0u8; 4096];
                if let Ok(n) = s.read(&mut buf) {
                    let head = String::from_utf8_lossy(&buf[..n]);
                    if head.contains(" 200 ") {
                        return true;
                    }
                }
            }
        }
        thread::sleep(Duration::from_millis(300));
    }
    false
}

fn start_adminvisit(root: &Path, port: u16) {
    let deno = root.join("runtime/deno/deno");
    let logf = fs::File::create(root.join("data/logs/adminvisit.out.log")).expect("log");
    let mut c = Command::new(&deno);
    c.arg("run").arg("--allow-all").arg(root.join("tools/adminvisit.js"));
    c.env(
        "OSEDE_ADMINVISIT_LOG",
        root.join("data/logs/adminvisit.log").to_string_lossy().to_string(),
    );
    c.env("OSEDE_BASE", format!("http://localhost:{}", port));
    c.stdin(Stdio::null());
    c.stdout(logf.try_clone().expect("clone"));
    c.stderr(logf);
    let child = c.spawn().expect("spawn deno");
    fs::write(root.join("data/run/adminvisit.pid"), child.id().to_string()).ok();
}

fn run(root: &Path, opts: &Opts) {
    if opts.install_service {
        install_service(root, opts);
        return;
    }
    if read_pid(root, "mysqld.pid").is_some()
        || read_pid(root, "nginx.pid").is_some()
        || read_pid(root, "php-fpm.pid").is_some()
    {
        eprintln!("something is already running, run ./osede stop first");
        std::process::exit(1);
    }
    ensure_dirs(root);
    fetch::ensure_runtimes(root);
    if port_in_use(3307) {
        let pids = find_pids("runtime/mysql/root/bin/mysqld");
        eprintln!(
            "error: port 3307 is already in use ({}); \
             a mysqld is already running - try ./osede stop first, \
             or kill the process manually",
            if pids.is_empty() {
                "unknown process".to_string()
            } else {
                format!(
                    "pids {}",
                    pids.iter().map(|p| p.to_string()).collect::<Vec<_>>().join(", ")
                )
            }
        );
        std::process::exit(1);
    }
    let datadir = root.join("data/mysql");
    let fresh = !datadir.exists()
        || fs::read_dir(&datadir)
            .map(|r| r.count() == 0)
            .unwrap_or(true);
    if fresh {
        println!("initializing mysql datadir (can take a minute on a slow VM)...");
        init_mysql(root);
    }
    println!("starting mysql...");
    start_mysql(root);
    if !wait_mysql(root) {
        eprintln!("mysql failed to start:");
        print_log_tail(root, "mysql-error.log", 15);
        std::process::exit(1);
    }
    if fresh {
        println!("creating database and users...");
        setup_db(root);
    }
    write_conf(root, opts);
    chown_web_dirs(root);
    println!("starting php-fpm...");
    start_fpm(root);
    if !wait_sock(root) {
        eprintln!("php-fpm failed to start:");
        print_log_tail(root, "php-fpm.log", 15);
        std::process::exit(1);
    }
    println!("starting nginx...");
    start_nginx(root);
    if !wait_http(opts.port) {
        eprintln!("web failed to start:");
        print_log_tail(root, "nginx-error.log", 15);
        std::process::exit(1);
    }
    if opts.adminvisit {
        start_adminvisit(root, opts.port);
    }
    println!("http://localhost:{} is up", opts.port);
}

fn status_cmd(root: &Path, port: u16) {
    for name in ["mysqld.pid", "php-fpm.pid", "nginx.pid", "adminvisit.pid"] {
        match read_pid(root, name) {
            Some(pid) => println!("{}: running (pid {})", name, pid),
            None => {
                if root.join("data/run").join(name).exists() {
                    println!("{}: dead (stale pid file)", name);
                } else {
                    println!("{}: stopped", name);
                }
            }
        }
    }
    let web = std::net::TcpStream::connect(format!("127.0.0.1:{}", port)).is_ok();
    println!(
        "web http 127.0.0.1:{}: {}",
        port,
        if web { "listening" } else { "closed" }
    );
    let my = std::net::TcpStream::connect("127.0.0.1:3307").is_ok();
    println!("mysql 127.0.0.1:3307: {}", if my { "listening" } else { "closed" });
}

fn reset(root: &Path) {
    stop_all(root);
    for d in ["data/mysql", "data/logs", "data/run", "data/sessions", "data/tmp", "data/conf"] {
        let p = root.join(d);
        fs::remove_dir_all(&p).ok();
        fs::create_dir_all(&p).ok();
    }
    for d in ["web/uploads", "web/logs"] {
        let p = root.join(d);
        if let Ok(rd) = fs::read_dir(&p) {
            for e in rd.flatten() {
                fs::remove_file(e.path()).ok();
            }
        }
        fs::create_dir_all(&p).ok();
    }
    println!("reset done, run ./osede run to start fresh");
}

fn systemctl_ok(args: &[&str]) -> bool {
    match Command::new("systemctl").args(args).output() {
        Ok(st) => st.status.success(),
        Err(_) => false,
    }
}

fn install_service(root: &Path, opts: &Opts) {
    if !systemctl_ok(&["--version"]) {
        eprintln!("error: systemctl not available, cannot install service");
        std::process::exit(1);
    }
    let exe = match env::current_exe().and_then(|p| fs::canonicalize(&p)) {
        Ok(p) => p,
        Err(_) => {
            eprintln!("error: cannot resolve path of the control binary");
            std::process::exit(1);
        }
    };
    let adminvisit = if opts.adminvisit { " --adminvisit" } else { "" };
    let run_cmd = format!(
        "\"{}\" run --port {} --ip {}{}",
        exe.display(),
        opts.port,
        opts.ip,
        adminvisit
    );
    let unit = format!(
        "[Unit]\n\
         Description=Osede Kommun - IT-security training environment\n\
         After=network-online.target\n\
         Wants=network-online.target\n\
         \n\
         [Service]\n\
         Type=oneshot\n\
         RemainAfterExit=yes\n\
         TimeoutStartSec=900\n\
         WorkingDirectory={}\n\
         ExecStart={}\n\
         ExecStop=\"{}\" stop\n\
         \n\
         [Install]\n\
         WantedBy=multi-user.target\n",
        root.display(),
        run_cmd,
        exe.display()
    );
    let unit_path = "/etc/systemd/system/osede.service";
    if let Err(e) = fs::write(unit_path, unit) {
        eprintln!("error: cannot write {} ({}), try running as root", unit_path, e);
        std::process::exit(1);
    }
    println!("wrote {}", unit_path);
    if !systemctl_ok(&["daemon-reload"]) || !systemctl_ok(&["enable", "osede"]) {
        eprintln!("error: systemctl daemon-reload / enable osede failed (try running as root)");
        std::process::exit(1);
    }
    let already = read_pid(root, "mysqld.pid").is_some()
        || read_pid(root, "nginx.pid").is_some()
        || read_pid(root, "php-fpm.pid").is_some();
    if already {
        println!(
            "service installed and enabled; the stack is already running and will be managed by the service after reboot (stop it first with ./osede stop to use the service now)"
        );
        return;
    }
    println!("starting service (first start downloads any missing runtime binaries)...");
    if !systemctl_ok(&["start", "osede"]) {
        eprintln!("error: systemctl start osede failed, check: journalctl -u osede");
        std::process::exit(1);
    }
    let start = Instant::now();
    let mut state = String::new();
    while start.elapsed() < Duration::from_secs(900) {
        if let Ok(o) = Command::new("systemctl").arg("is-active").arg("osede").output() {
            state = String::from_utf8_lossy(&o.stdout).trim().to_string();
            if state == "active" || state == "failed" {
                break;
            }
        }
        thread::sleep(Duration::from_secs(2));
    }
    if state == "active" {
        println!("service started, http://localhost:{} is up", opts.port);
    } else {
        eprintln!(
            "service did not start (state: {}), check: journalctl -u osede",
            state
        );
        std::process::exit(1);
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let root = root_dir();
    let opts = parse_opts(args.get(2..).unwrap_or(&[]));
    match args.get(1).map(|s| s.as_str()).unwrap_or("") {
        "run" => run(&root, &opts),
        "stop" => {
            stop_all(&root);
            println!("stopped");
        }
        "status" => status_cmd(&root, opts.port),
        "reset" => reset(&root),
        "fetch" => {
            fetch::ensure_runtimes(&root);
            println!("runtime components up to date");
        }
        _ => {
            println!("usage: osede [run [--port N] [--ip ADDR] [--adminvisit] [--install-service] | stop | status [--port N] | reset | fetch]");
        }
    }
}

use std::fs::{self, File};
use std::io::{Read, Write};
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

use flate2::read::GzDecoder;
use indicatif::{ProgressBar, ProgressStyle};
use sha2::{Digest, Sha256};
use tar::Archive;

const DEFAULT_BASE: &str =
    "https://github.com/jra89/osedekommun/releases/download/v1.0.0";

pub struct Component {
    pub name: &'static str,
    /// Relative path from the project root. If this file exists and is
    /// executable, the component is considered present.
    pub marker: &'static str,
}

pub const COMPONENTS: &[Component] = &[
    Component {
        name: "mysql",
        marker: "runtime/mysql/root/bin/mysqld",
    },
    Component {
        name: "php",
        marker: "runtime/php/sbin/php-fpm",
    },
    Component {
        name: "nginx",
        marker: "runtime/nginx/sbin/nginx",
    },
    Component {
        name: "deno",
        marker: "runtime/deno/deno",
    },
];

fn base_url() -> String {
    std::env::var("OSEDE_RUNTIME_BASE")
        .ok()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| DEFAULT_BASE.to_string())
}

fn is_present(root: &Path, marker: &str) -> bool {
    let p = root.join(marker);
    match p.symlink_metadata() {
        Ok(md) => md.is_file() && md.permissions().mode() & 0o100 != 0,
        Err(_) => false,
    }
}

fn sha256_of(path: &Path) -> Result<String, String> {
    let mut file = File::open(path).map_err(|e| e.to_string())?;
    let mut hasher = Sha256::new();
    let mut buf = [0u8; 65536];
    loop {
        let n = file.read(&mut buf).map_err(|e| e.to_string())?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

fn download_manifest(base: &str) -> Result<Vec<(String, String)>, String> {
    let url = format!("{}/SHA256SUMS", base);
    let resp = ureq::get(&url)
        .call()
        .map_err(|e| format!("cannot download {}:\n  {}", url, e))?;
    let mut text = String::new();
    resp.into_reader()
        .read_to_string(&mut text)
        .map_err(|e| format!("cannot read {}: {}", url, e))?;
    let mut sums = Vec::new();
    for line in text.lines() {
        let mut it = line.split_whitespace();
        if let (Some(hash), Some(name)) = (it.next(), it.next()) {
            sums.push((
                name.trim_start_matches("./").to_string(),
                hash.to_ascii_lowercase(),
            ));
        }
    }
    if sums.is_empty() {
        return Err(format!("{} contains no checksums", url));
    }
    Ok(sums)
}

fn download(base: &str, file: &str, dest: &Path, sums: &[(String, String)]) -> Result<(), String> {
    let url = format!("{}/{}", base, file);
    let tmp = dest.with_extension("part");
    let resp = match ureq::get(&url).call() {
        Ok(r) => r,
        Err(e) => {
            fs::remove_file(&tmp).ok();
            return Err(format!(
                "cannot download {}\n  {}\n  Check that the release assets exist, or point OSEDE_RUNTIME_BASE at the correct location.",
                url, e
            ));
        }
    };
    let total = resp
        .header("content-length")
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);
    let pb = if total > 0 {
        let pb = ProgressBar::new(total);
        pb.set_style(
            ProgressStyle::with_template(
                "{prefix} [{bar:40}] {percent}% {bytes}/{total_bytes} ({bytes_per_sec})",
            )
            .unwrap()
            .progress_chars("#>-"),
        );
        pb.set_prefix(file.to_string());
        pb
    } else {
        let pb = ProgressBar::new(u64::MAX);
        pb.set_style(ProgressStyle::with_template("{prefix} {bytes} ({bytes_per_sec})").unwrap());
        pb.set_prefix(file.to_string());
        pb
    };
    let mut reader = resp.into_reader();
    let mut out = File::create(&tmp).map_err(|e| e.to_string())?;
    let mut buf = [0u8; 65536];
    loop {
        let n = match reader.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => n,
            Err(e) => {
                fs::remove_file(&tmp).ok();
                return Err(format!("download failed for {}: {}", url, e));
            }
        };
        if let Err(e) = out.write_all(&buf[..n]) {
            fs::remove_file(&tmp).ok();
            return Err(e.to_string());
        }
        pb.inc(n as u64);
    }
    pb.finish_with_message(format!("{} downloaded and verified", file));
    if let Some((_, hash)) = sums.iter().find(|(n, _)| n == file) {
        let actual = sha256_of(&tmp)?;
        if actual != *hash {
            fs::remove_file(&tmp).ok();
            return Err(format!(
                "checksum mismatch for {}\n  expected {}\n  actual   {}",
                file, hash, actual
            ));
        }
    }
    fs::rename(&tmp, dest).map_err(|e| e.to_string())?;
    Ok(())
}

fn extract(tarball: &Path, root: &Path) -> Result<(), String> {
    let file = File::open(tarball).map_err(|e| e.to_string())?;
    let gz = GzDecoder::new(file);
    let mut ar = Archive::new(gz);
    fs::create_dir_all(root.join("runtime")).map_err(|e| e.to_string())?;
    ar.unpack(root.join("runtime"))
        .map_err(|e| format!("failed to extract {}: {}", tarball.display(), e))
}

fn ensure_exec(path: &Path) {
    if let Ok(md) = fs::metadata(path) {
        let mode = md.permissions().mode();
        if mode & 0o100 == 0 {
            let mut p = md.permissions();
            p.set_mode(mode | 0o755);
            fs::set_permissions(path, p).ok();
        }
    }
}

pub fn ensure_runtimes(root: &Path) {
    let tmp = root.join("data/tmp");
    fs::create_dir_all(&tmp).ok();
    let missing: Vec<&Component> = COMPONENTS
        .iter()
        .filter(|c| !is_present(root, c.marker))
        .collect();
    if missing.is_empty() {
        return;
    }
    println!(
        "downloading missing runtime components: {}",
        missing
            .iter()
            .map(|c| c.name)
            .collect::<Vec<_>>()
            .join(", ")
    );
    let base = base_url();
    let sums = match download_manifest(&base) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("error: {}", e);
            std::process::exit(1);
        }
    };
    for c in &missing {
        let file = format!("runtime-{}.tar.gz", c.name);
        let dest = tmp.join(&file);
        if let Err(e) = download(&base, &file, &dest, &sums) {
            eprintln!("error: {}", e);
            std::process::exit(1);
        }
        if let Err(e) = extract(&dest, root) {
            eprintln!("error: {}", e);
            std::process::exit(1);
        }
        ensure_exec(&root.join(c.marker));
        println!("{} ready", c.name);
        fs::remove_file(&dest).ok();
    }
}

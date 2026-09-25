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

/// (major, minor) of the glibc this process runs on, e.g. (2, 39).
fn local_glibc() -> Option<(u32, u32)> {
    unsafe {
        let ptr = libc::gnu_get_libc_version();
        if ptr.is_null() {
            return None;
        }
        let s = std::ffi::CStr::from_ptr(ptr).to_str().ok()?;
        let mut it = s.split('.');
        let major: u32 = it.next()?.parse().ok()?;
        let minor: u32 = it.next()?.parse().ok()?;
        Some((major, minor))
    }
}

/// Parse a tag like "231" into 231 (2.31 * 100) for comparison.
fn glibc_tag_value(tag: &str) -> Option<u64> {
    if tag.len() < 3 || !tag.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    let major: u64 = tag[..tag.len() - 2].parse().ok()?;
    let minor: u64 = tag[tag.len() - 2..].parse().ok()?;
    Some(major * 100 + minor)
}

/// Pick the release asset for a component: the newest
/// `runtime-<comp>-glibc<Mmm>.tar.gz` whose glibc requirement is <= the local
/// one, else the untagged `runtime-<comp>.tar.gz` (official builds that run
/// on any glibc).
fn pick_asset(sums: &[(String, String)], comp: &str, local: Option<(u32, u32)>) -> Option<String> {
    let prefix = format!("runtime-{}-glibc", comp);
    let local_val = local.map(|(m, n)| (m as u64) * 100 + n as u64);
    let mut best: Option<(u64, String)> = None;
    for (name, _) in sums {
        let tag = match name.strip_prefix(&prefix).and_then(|t| t.strip_suffix(".tar.gz")) {
            Some(t) => t,
            None => continue,
        };
        let value = match glibc_tag_value(tag) {
            Some(v) => v,
            None => continue,
        };
        if Some(value) > local_val {
            continue;
        }
        if best.as_ref().map(|(v, _)| value > *v).unwrap_or(true) {
            best = Some((value, name.clone()));
        }
    }
    if let Some((_, name)) = best {
        return Some(name);
    }
    let plain = format!("runtime-{}.tar.gz", comp);
    if sums.iter().any(|(n, _)| n == &plain) {
        return Some(plain);
    }
    None
}

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
    let local = local_glibc();
    if let Some((major, minor)) = local {
        println!("local glibc: {}.{}", major, minor);
    } else {
        eprintln!("warning: could not detect local glibc, falling back to untagged runtime assets");
    }
    for c in &missing {
        let file = match pick_asset(&sums, c.name, local) {
            Some(f) => f,
            None => {
                eprintln!(
                    "error: no suitable runtime-{} asset found in the manifest \
                     (need a build for glibc <= {})",
                    c.name,
                    local
                        .map(|(m, n)| format!("{}.{n}", m))
                        .unwrap_or_else(|| "unknown".into())
                );
                std::process::exit(1);
            }
        };
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

#[cfg(test)]
mod tests {
    use super::*;

    fn sums(names: &[&str]) -> Vec<(String, String)> {
        names.iter().map(|n| (n.to_string(), "x".to_string())).collect()
    }

    #[test]
    fn picks_newest_variant_that_fits() {
        let s = sums(&[
            "runtime-php.tar.gz",
            "runtime-php-glibc231.tar.gz",
            "runtime-php-glibc239.tar.gz",
        ]);
        assert_eq!(
            pick_asset(&s, "php", Some((2, 39))),
            Some("runtime-php-glibc239.tar.gz".into())
        );
        assert_eq!(
            pick_asset(&s, "php", Some((2, 31))),
            Some("runtime-php-glibc231.tar.gz".into())
        );
    }

    #[test]
    fn falls_back_to_untagged() {
        let s = sums(&["runtime-mysql.tar.gz", "runtime-mysql-glibc239.tar.gz"]);
        assert_eq!(
            pick_asset(&s, "mysql", Some((2, 31))),
            Some("runtime-mysql.tar.gz".into())
        );
        let s = sums(&["runtime-mysql.tar.gz"]);
        assert_eq!(
            pick_asset(&s, "mysql", Some((9, 99))),
            Some("runtime-mysql.tar.gz".into())
        );
    }

    #[test]
    fn no_suitable_variant() {
        let s = sums(&["runtime-nginx-glibc239.tar.gz"]);
        assert_eq!(pick_asset(&s, "nginx", Some((2, 31))), None);
        assert_eq!(pick_asset(&s, "nginx", None), None);
    }

    #[test]
    fn ignores_malformed_tags() {
        let s = sums(&[
            "runtime-php-glibcx.tar.gz",
            "runtime-php-glibc2.tar.gz",
            "runtime-php-glibc2311.tar.gz",
            "runtime-php-glibc231.tar.gz",
        ]);
        assert_eq!(
            pick_asset(&s, "php", Some((2, 31))),
            Some("runtime-php-glibc231.tar.gz".into())
        );
    }

    #[test]
    fn component_prefix_does_not_cross_match() {
        let s = sums(&["runtime-php-glibc231.tar.gz"]);
        assert_eq!(pick_asset(&s, "nginx", Some((2, 31))), None);
    }
}

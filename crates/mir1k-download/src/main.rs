use std::io::{Cursor, Read, Write};
use std::path::Path;

const DOWNLOAD_URL: &str =
    "https://huggingface.co/datasets/AnhP/Mir-1k-use-DJCM-training/resolve/main/MIR-1K.zip";

const TARGET_DIR: &str = "external-samples/MIR-1K";

fn main() -> Result<(), String> {
    let zip_data = download(DOWNLOAD_URL)?;
    extract(&zip_data, TARGET_DIR)?;
    Ok(())
}

fn download(url: &str) -> Result<Vec<u8>, String> {
    eprint!("Downloading MIR-1K.zip ... ");
    let resp = ureq::get(url)
        .call()
        .map_err(|e| format!("HTTP request failed: {e}"))?;

    let total: u64 = resp
        .headers()
        .get("content-length")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);

    let mut body = Vec::new();
    let mut reader = resp.into_body().into_reader();
    let mut downloaded: u64 = 0;
    let mut buf = [0u8; 65536];

    loop {
        let n = reader
            .read(&mut buf)
            .map_err(|e| format!("read error: {e}"))?;
        if n == 0 {
            break;
        }
        body.write_all(&buf[..n])
            .map_err(|e| format!("buffer write: {e}"))?;
        downloaded += n as u64;
        if total > 0 {
            let pct = downloaded as f64 / total as f64 * 100.0;
            eprint!("\rDownloading MIR-1K.zip ... {pct:.0}%");
        }
    }
    eprintln!();
    Ok(body)
}

fn extract(zip_data: &[u8], target: &str) -> Result<(), String> {
    let target = Path::new(target);
    let mut archive =
        zip::ZipArchive::new(Cursor::new(zip_data)).map_err(|e| format!("zip open: {e}"))?;

    let mut file_count = 0u32;
    for i in 0..archive.len() {
        let mut entry = archive
            .by_index(i)
            .map_err(|e| format!("entry {i}: {e}"))?;
        let Some(path) = entry.enclosed_name() else {
            continue;
        };

        let out_path = target.join(path);

        if entry.is_dir() {
            std::fs::create_dir_all(&out_path)
                .map_err(|e| format!("mkdir {:?}: {e}", out_path))?;
        } else {
            if let Some(parent) = out_path.parent() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| format!("mkdir {:?}: {e}", parent))?;
            }
            let mut out = std::fs::File::create(&out_path)
                .map_err(|e| format!("create {:?}: {e}", out_path))?;
            std::io::copy(&mut entry, &mut out)
                .map_err(|e| format!("write {:?}: {e}", out_path))?;
            file_count += 1;
        }
    }

    println!(
        "Extracted {file_count} files to {}",
        target
            .canonicalize()
            .unwrap_or_else(|_| target.to_path_buf())
            .display()
    );
    Ok(())
}

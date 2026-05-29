//! Archive extraction — ZIP and tar.gz with Zip Slip protection.

use crate::Error;
use flate2::read::GzDecoder;
use std::fs;
use std::path::Path;

pub fn extract(archive_path: &Path, out_dir: &Path) -> Result<(), Error> {
    let fname = archive_path.to_string_lossy();
    if fname.ends_with(".tar.gz") || fname.ends_with(".tgz") {
        extract_tar_gz(archive_path, out_dir)
    } else {
        extract_zip(archive_path, out_dir)
    }
}

fn extract_tar_gz(archive_path: &Path, out_dir: &Path) -> Result<(), Error> {
    let file = fs::File::open(archive_path)?;
    let decoder = GzDecoder::new(file);
    let mut archive = tar::Archive::new(decoder);
    // Reject entries with absolute paths or .. components
    archive.set_overwrite(false);

    for entry in archive
        .entries()
        .map_err(|e| Error::Extract(e.to_string()))?
    {
        let mut entry = entry.map_err(|e| Error::Extract(e.to_string()))?;
        let entry_path = entry
            .path()
            .map_err(|e| Error::Extract(e.to_string()))?
            .into_owned();

        // Reject path traversal attempts before extraction
        if entry_path.is_absolute()
            || entry_path
                .components()
                .any(|c| c == std::path::Component::ParentDir)
        {
            return Err(Error::Extract(format!(
                "Path traversal detected: {}",
                entry_path.display()
            )));
        }

        // unpack_in does its own traversal check as a second layer
        entry
            .unpack_in(out_dir)
            .map_err(|e| Error::Extract(e.to_string()))?;
    }
    Ok(())
}

fn extract_zip(archive_path: &Path, out_dir: &Path) -> Result<(), Error> {
    let file = fs::File::open(archive_path)?;
    let mut archive = zip::ZipArchive::new(file).map_err(|e| Error::Extract(e.to_string()))?;

    for i in 0..archive.len() {
        let mut entry = archive
            .by_index(i)
            .map_err(|e| Error::Extract(e.to_string()))?;
        let entry_name = entry.name().to_string();

        // Zip Slip protection — reject paths with .. components
        if entry_name.contains("..") {
            return Err(Error::Extract(format!("Zip Slip detected: {entry_name}")));
        }

        let dest = out_dir.join(&entry_name);
        if !dest.starts_with(out_dir) {
            return Err(Error::Extract(format!(
                "Path traversal detected: {entry_name}"
            )));
        }

        if entry.is_dir() {
            fs::create_dir_all(&dest)?;
        } else {
            if let Some(parent) = dest.parent() {
                fs::create_dir_all(parent)?;
            }
            let mut outfile = fs::File::create(&dest)?;
            std::io::copy(&mut entry, &mut outfile)?;

            // Preserve Unix permissions
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                if let Some(mode) = entry.unix_mode() {
                    fs::set_permissions(&dest, fs::Permissions::from_mode(mode))?;
                }
            }
        }
    }
    Ok(())
}

use std::{fs, io};
use std::path::Path;

use walkdir::WalkDir;



pub fn create_dir_no_check<P: AsRef<Path>>(path: P) -> bool{
    if path.as_ref().exists() || path.as_ref().is_file() {
        return true;
    }
    let mut res = true;
    fs::create_dir(path).unwrap_or_else(|e| {
        eprintln!("Error creating directory: {}", e);
        res = false;
    });
    res
}

pub fn move_file<P: AsRef<Path>, Q: AsRef<Path>>(src: P, dst: Q) -> bool {
    let mut res = true;
    fs::rename(src, dst).unwrap_or_else(|e| {
        eprintln!("Error moving file: {}", e);
        res = false;
    });
    res
}


pub fn copy_dirs<P: AsRef<Path>>(src: P, dst: P) -> io::Result<()> {
    if let Some(filename) = src.as_ref().file_name() {
        let dest_path = dst.as_ref().join(filename);
        println!("Copying merged mod directory to: {}", &dest_path.display());
        copy_dir(&src, &dest_path)?;
    } else {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "Couldn't find a directory to copy",
        ));
    }
    Ok(())
}

fn copy_dir<P: AsRef<Path>, Q: AsRef<Path>>(source_path: P, dest_path: Q) -> io::Result<()> {
    let dst = dest_path.as_ref();
    let src = source_path.as_ref();
    if !dst.exists() {
        fs::create_dir_all(dst)?;
    }

    for entry in WalkDir::new(src) {
        let entry = entry?;
        let src_path = entry.path();
        let rel_path = src_path.strip_prefix(src).map_err(|e| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Couldn't strip prefix: {}", e),
            )
        })?;
        let dst_path = dst.join(rel_path);

        if src_path.is_dir() {
            fs::create_dir_all(&dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)?;
        }
    }
    Ok(())
}
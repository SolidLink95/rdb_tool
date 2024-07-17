use std::{fs, io};
use std::path::{Path, PathBuf};

use walkdir::WalkDir;

pub fn is_yuzu_dir<P: AsRef<Path>>(path: P) -> bool {
    let mut p = PathBuf::from(path.as_ref());
    if !p.is_dir() {
        return false;
    }
    let mut game_id = String::new();
    let mut yuzu_dir = String::new();
    let mut load_dir = String::new();

    if let Some(filename) = p.file_name() {
        game_id = filename.to_str().unwrap_or_default().to_string().to_uppercase();
    } 
    if !p.pop() {
        return false;
    }
    if let Some(filename) = p.file_name() {
        load_dir = filename.to_str().unwrap_or_default().to_string();
    } 
    if !p.pop() {
        return false;
    }
    if let Some(filename) = p.file_name() {
        yuzu_dir = filename.to_str().unwrap_or_default().to_string();
    } 


    return game_id == "01002B00111A2000" && load_dir == "load" && yuzu_dir == "yuzu";
}
pub fn is_ryu_dir<P: AsRef<Path>>(path: P) -> bool {
    let mut p = PathBuf::from(path.as_ref());
    if !p.is_dir() {
        return false;
    }
    let mut game_id = String::new();
    let mut ryu_dir = String::new();

    if let Some(filename) = p.file_name() {
        game_id = filename.to_str().unwrap_or_default().to_string().to_uppercase();
    } 
    for _ in 0..3  {
        if !p.pop() {
            return false;
        }
    }
    if let Some(filename) = p.file_name() {
        ryu_dir = filename.to_str().unwrap_or_default().to_string();
    } 

    return game_id == "01002B00111A2000"  && ryu_dir == "Ryujinx";
}

pub fn is_emulator_dir<P:AsRef<Path>>(path: P) -> String {
    if is_yuzu_dir(&path) {
        return "Yuzu".to_string();
    }
    if is_ryu_dir(&path) {
        return "Ryujinx".to_string();
    }
    return String::new();
}

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
use io::Error as ioErr;
use io::ErrorKind as ErrKind;
use std::{
    collections::HashMap,
    fmt, fs, io,
    path::{Path, PathBuf},
    process,
    sync::Arc,
};

use crate::{
    rdb::{self, Rdb},
    utils::*,
    AocConfig::{AocConfig, Pathlib},
};

#[derive(Debug, Clone)]
pub struct ModDir {
    pub path: PathBuf,
    pub rdb_path: PathBuf,
    pub data_path: PathBuf,
    pub patch_path: PathBuf,
    pub add_paths: Vec<PathBuf>,
}

impl Default for ModDir {
    fn default() -> Self {
        Self {
            path: PathBuf::default(),
            rdb_path: PathBuf::default(),
            data_path: PathBuf::default(),
            patch_path: PathBuf::default(),
            add_paths: Vec::new(),
        }
    }
}

impl ModDir {
    pub fn new<P: AsRef<Path>>(path: P, add_paths: Vec<PathBuf>, create_dirs: bool) -> Self {
        let p = PathBuf::from(path.as_ref());
        let mut rdb_path = p.clone();
        rdb_path.push("romfs/asset");
        let mut data_path = p.clone();
        data_path.push("romfs/asset/data");
        let mut patch_path = p.clone();
        let new_add_paths = Vec::new();
        patch_path.push("romfs/asset/patch");
        if create_dirs {
            create_dir_no_check(&patch_path);
            create_dir_no_check(&data_path);
        }
        for add_path in add_paths.iter() {
            let mut new_add_path = p.clone();
            new_add_path.push(add_path);
            if create_dirs {
                create_dir_no_check(new_add_path);
            }
        }
        Self {
            path: p,
            rdb_path: rdb_path,
            data_path: data_path,
            patch_path: patch_path,
            add_paths: new_add_paths,
        }
    }
    pub fn remove_self_if_exists(&self) -> io::Result<()> {
        if self.path.exists() {
            fs::remove_dir_all(&self.path)?;
        }
        Ok(())
    }

    pub fn create_dirs_all(&self) -> io::Result<()> {
        fs::create_dir_all(&self.path)?;
        fs::create_dir_all(&self.data_path)?;
        fs::create_dir_all(&self.patch_path)?;
        for add_path in self.add_paths.iter() {
            fs::create_dir_all(add_path)?;
        }
        Ok(())
    }
}

pub struct ModMerger {
    pub config: Arc<AocConfig>,
    pub root_mod_name: String,
    pub cwd_dir: String,
    pub root_dir: ModDir,
    pub mods_dirs: Vec<ModDir>,
    pub add_paths: Vec<String>,
    // pub rdbs: HashMap<String, Rdb>,
    pub aoc_hashes: HashMap<String, Vec<AocHash>>,
}

impl ModMerger {
    pub fn new_default() -> io::Result<Self> {
        Ok(Self {
            config: Arc::new(AocConfig::new()?),
            root_mod_name: "000_AOC_MERGED_MODS".to_string(),
            cwd_dir: Default::default(),
            root_dir: Default::default(),
            mods_dirs: Vec::new(),
            add_paths: vec!["exefs".to_string(), "romfs/movie_logo".to_string()],
            // rdbs: HashMap::new(),
            aoc_hashes: Default::default(),
        })
    }
    pub fn new<P: AsRef<Path>>(cwd_dir: Option<P>) -> io::Result<Self> {
        let mut rdir = String::new();
        if let Some(p) = cwd_dir {
            rdir = p.as_ref().to_string_lossy().to_string();
        } else {
            rdir = std::env::current_dir()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
        }
        let mut res = Self::new_default()?;
        let mut add_paths = Vec::new();
        for add_path in res.add_paths.iter() {
            let mut p = PathBuf::from(&rdir);
            p.push(add_path);
            add_paths.push(p);
        }
        res.cwd_dir = rdir.clone();
        res.root_dir = ModDir::new(Path::new(&rdir).join(&res.root_mod_name), add_paths, false);
        res.root_dir.remove_self_if_exists()?;
        res.root_dir.create_dirs_all()?;
        // res.get_mods_dirs()?;
        Ok(res)
    }

    pub fn process_mods(&mut self) -> io::Result<()> {
        let rdb_dest_dir = self.root_dir.rdb_path.clone();
        // println!("{}:{}: rdb_dest_dir {:?}", file!(), line!(), &rdb_dest_dir);

        for entry in fs::read_dir(&self.cwd_dir)? {
            let path = Pathlib::new(entry?.path());
            if path.is_dir()
                && path.name != self.root_mod_name
                && self.is_valid_mod_dir(&path.full_path)
            {
                self.mods_dirs.push(ModDir::new(&path.full_path, self.root_dir.add_paths.clone(), false));
            }
        }
        self.mods_dirs.sort_by(|a, b| {
            a.path.to_string_lossy()
                .to_lowercase()
                .cmp(&b.path.to_string_lossy().to_lowercase())
        });
        println!("Age Of Calamity Mods Merger 1.0\nMerging {} mods\n\n", self.mods_dirs.len());
        for mod_dir in self.mods_dirs.clone().iter().rev().cloned() {
            println!("Processing mod directory: {}", mod_dir.path.display());
            self.copy_add_paths(&mod_dir)?;
            self.update_aoc_hashes_from_modpath(mod_dir)?;
        }
        println!("\n\n");
        // println!("{}:{}: aoc_hashes {:?}", file!(), line!(), &self.aoc_hashes);

        for (rdb_name, hashes) in self.aoc_hashes.iter_mut() {
            if !hashes.is_empty() {
                if let Some(rdb_path) = self.config.get_rdb_path(&rdb_name) {
                    let mut rdb = Rdb::open_io(rdb_path)?;
                    println!("Starting to patch {}", rdb_name);
                    let mut processed_hashes:Vec<&str> = Vec::new();
                    for aoc_hash in hashes.iter_mut() {
                        let filename = &aoc_hash.as_hex_str();
                        if processed_hashes.contains(&&aoc_hash.hash.as_str()) {
                            continue;
                        }
                        match rdb.get_entry_by_ktid_mut(crate::ktid(filename)) {
                            Some(entry_found) => {
                                print!("Patching {} ... ", &aoc_hash.path.name);
                                entry_found.make_external();
                                entry_found.make_uncompressed();
                                let destpath = self.root_dir.data_path.join(format!("0x{}.file", &aoc_hash.hash));
                                if let Ok(rawdata) = entry_found.set_external_file(&aoc_hash) {
                                    println!("Entry converted nicely");
                                    fs::write(&destpath, &rawdata)?;
                                } else {
                                    //assuming the file needs to be copied
                                    if !destpath.exists() {
                                        println!("Entry already converted, copying");
                                        fs::copy(&aoc_hash.path.full_path, &destpath)?;
                                    }
                                }
                            }
                            None => println!("File {} not found in the RDB. Skipping.", filename),
                        }
                        processed_hashes.push(&aoc_hash.hash);
                    }
                    let rdb_dest_path = rdb_dest_dir.join(rdb_name);
                    rdb.save(&rdb_dest_path)?;
                }
            } else {
                eprintln!("ERROR: RDB not found for {}", rdb_name);
            }
            println!("\n\n");
        }

        Ok(())
    }

    pub fn copy_add_paths(&self, mod_dir: &ModDir) ->io::Result<()> {
        for add_path in self.add_paths.iter() {
            let source_path = PathBuf::from(&mod_dir.path).join(add_path);
            if source_path.exists() {
                let destpath = PathBuf::from(&self.root_dir.path).join(add_path);
                if !destpath.exists() {
                    fs::create_dir_all(&destpath)?;
                }
                for entry in fs::read_dir(&source_path)? {
                    let entry = entry?;
                    let path: PathBuf = entry.path();
                    if let Some(filename) = path.file_name() {
                        let dest_file = destpath.join(filename);
                        fs::copy(&path, &dest_file)?;
                    } else {
                        eprintln!("ERROR: Invalid file name: {:?}", path);
                    
                    }
                }
            }
        }

        Ok(())
    }

    pub fn update_aoc_hashes_from_modpath(
        &mut self,
        mod_path: ModDir,
    ) -> io::Result<()> {
        if let Ok(entries) = fs::read_dir(&mod_path.data_path) {
            for entry in entries {
                if let Ok(entry) = entry {
                    let path = entry.path();
                    if path.is_file() {
                        let aoc_hash = AocHash::new(&path, self.config.clone());
                        if let Some(rdb_name) = &aoc_hash.rdb_name {
                            if !self.aoc_hashes.contains_key(rdb_name) {
                                self.aoc_hashes.insert(rdb_name.to_string(), Vec::new());
                            }
                            if let Some(v) = self.aoc_hashes.get_mut(rdb_name) {
                                v.push(aoc_hash);
                            }
                        } else {
                            eprintln!("ERROR: Invalid hash, no rdb found: {:?}", aoc_hash);
                        }
                    }
                }
            }
        }

        Ok(())
    }

    // pub fn get_mods_dirs(&mut self) -> io::Result<()> {
    //     for entry in fs::read_dir(&self.cwd_dir)? {
    //         let path = Pathlib::new(entry?.path());
    //         if path.is_dir()
    //             && path.name != self.root_mod_name
    //             && self.is_valid_mod_dir(&path.full_path)
    //         {
    //             self.mods_dirs.push(PathBuf::from(&path.full_path));
    //         }
    //     }

    //     Ok(())
    // }

    pub fn get_rdb_name(&self, hash: &AocHash) -> Option<String> {
        for (rdb_name, hashes) in self.config.hashes.iter() {
            if hashes.contains(&hash.hash) {
                return Some(rdb_name.to_string());
            }
        }
        None
    }

    pub fn is_valid_mod_dir<P: AsRef<Path>>(&self, path: P) -> bool {
        let p = PathBuf::from(path.as_ref());
        if let Some(name) = p.file_name() {
            if name.to_string_lossy().to_string().starts_with("#") {
                return false;
            }
        }
        if p.parent().unwrap().starts_with("#") {
            return false;
        }
        p.exists() && p.is_dir()
    }

    pub fn prepare_master_dir(&mut self) -> io::Result<PathBuf> {
        let mut p = PathBuf::from(&self.cwd_dir);
        p.push(&self.root_mod_name);
        if p.exists() {
            std::fs::remove_dir_all(&p)?;
        }
        p.push("romfs/asset/data");
        std::fs::create_dir_all(&p)?;
        p.pop();
        Ok(p)
    }
}

#[derive(Clone)]
pub struct AocHash {
    pub path: Pathlib,
    pub hash: String,
    pub rdb_name: Option<String>,
}

impl AocHash {
    pub fn new<P: AsRef<Path>>(path: P, config: Arc<AocConfig>) -> Self {
        let p = Pathlib::new(path);
        let hash = p.stem.to_string().to_lowercase().replace("0x", "");
        let rdb_name = config.hashes_rev.get(&hash).map(|x| x.to_string());
        Self {
            path: p,
            hash: hash,
            rdb_name: rdb_name,
        }
    }

    pub fn copy_if_needed(&mut self) -> io::Result<()> {
        if !Path::new(&self.path.full_path).exists() {
            return Err(ioErr::new(
                ErrKind::NotFound,
                format!("File not found: {}", &self.path.full_path),
            ));
        }
        let mut new_path = PathBuf::from(&self.path.full_path);
        new_path.pop();
        new_path.push(format!("{}.file", &self.as_hex_str()));
        if !new_path.exists() {
            fs::rename(&self.path.full_path, &new_path)?;
            self.path = Pathlib::new(&new_path);
        }
        Ok(())
    }

    pub fn is_valid(&self) -> bool {
        if self.rdb_name.is_none() {
            return false;
        }
        if self.hash.starts_with("0x") {
            return u32::from_str_radix(&self.hash[2..], 16).is_ok();
        }
        u32::from_str_radix(&self.hash, 16).is_ok()
    }

    pub fn as_u32(&self) -> io::Result<u32> {
        u32::from_str_radix(&self.hash, 16)
            .map_err(|_| ioErr::new(ErrKind::InvalidData, "Invalid hash"))
    }

    pub fn as_hex_str(&self) -> String {
        if self.hash.starts_with("0x") {
            self.hash.to_string()
        } else {
            format!("0x{}", self.hash)
        }
    }
}

impl fmt::Debug for AocHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AocHash")
            .field("full_path", &self.path.full_path)
            .finish()
    }
}

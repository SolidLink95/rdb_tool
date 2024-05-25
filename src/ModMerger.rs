use std::{
    collections::HashMap,
    fmt, fs, io,
    path::{Path, PathBuf},
    process,
    sync::Arc,
};

use crate::{
    rdb::{self, Rdb},
    AocConfig::{AocConfig, Pathlib},
};

pub struct ModMerger {
    pub config: Arc<AocConfig>,
    pub root_mod_name: String,
    pub root_dir: String,
    pub mods_dirs: Vec<PathBuf>,
    // pub rdbs: HashMap<String, Rdb>,
    pub aoc_hashes: HashMap<String, Vec<AocHash>>,
}

impl ModMerger {
    pub fn new_default() -> io::Result<Self> {
        Ok(Self {
            config: Arc::new(AocConfig::new()?),
            root_mod_name: "000_AOC_MERGED_MODS".to_string(),
            root_dir: Default::default(),
            mods_dirs: Vec::new(),
            // rdbs: HashMap::new(),
            aoc_hashes: Default::default(),
        })
    }
    pub fn new<P: AsRef<Path>>(root_dir: Option<P>) -> io::Result<Self> {
        let mut rdir = String::new();
        if let Some(p) = root_dir {
            rdir = p.as_ref().to_string_lossy().to_string();
        } else {
            rdir = std::env::current_dir()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
        }
        let mut res = Self::new_default()?;
        res.root_dir = rdir;
        // res.get_mods_dirs()?;

        Ok(res)
    }

    pub fn process_mods(&mut self) -> io::Result<()> {
        let rdb_dest_dir = self.prepare_master_dir()?;
        // println!("{}:{}: rdb_dest_dir {:?}", file!(), line!(), &rdb_dest_dir);

        for entry in fs::read_dir(&self.root_dir)? {
            let path = Pathlib::new(entry?.path());
            if path.is_dir()
                && path.name != self.root_mod_name
                && self.is_valid_mod_dir(&path.full_path)
            {
                self.mods_dirs.push(PathBuf::from(&path.full_path));
            }
        }
        self.mods_dirs.sort_by(|a, b| {
            a.to_string_lossy()
                .to_lowercase()
                .cmp(&b.to_string_lossy().to_lowercase())
        });
        // println!(
        //     "{}:{}: mods_dirs (len {}) {:?}",
        //     file!(),
        //     line!(),
        //     &self.mods_dirs.len(),
        //     &self.mods_dirs
        // );
        for mod_dir in self.mods_dirs.clone().iter().rev().cloned() {
            self.update_aoc_hashes_from_modpath(mod_dir)?;
        }
        // println!("{}:{}: aoc_hashes {:?}", file!(), line!(), &self.aoc_hashes);

        for (rdb_name, hashes) in self.aoc_hashes.iter() {
            if !hashes.is_empty() {
                if let Some(rdb_path) = self.config.get_rdb_path(&rdb_name) {
                    let mut rdb = Rdb::open_io(rdb_path)?;
                    println!("Starting to patch {}", rdb_name);
                    for aoc_hash in hashes.iter() {
                        let filename = &aoc_hash.as_hex_str();
                        let entry_path = PathBuf::from(&aoc_hash.path.full_path);
                        match rdb.get_entry_by_ktid_mut(crate::ktid(filename)) {
                            Some(entry_found) => {
                                print!("Patching {}... ", filename);
                                entry_found.make_external();
                                entry_found.make_uncompressed();
                                if let Ok(_) = entry_found.set_external_file(&entry_path) {
                                    println!("Entry converted nicely");
                                } else {
                                    println!("Entry already converted, skipping");
                                }
                                // entry_found
                                //     .set_external_file(&entry.path())
                                //     .expect("Failed to set external file");
                            }
                            None => println!("File {} not found in the RDB. Skipping.", filename),
                        }
                    }
                    let mut rdb_dest_path = rdb_dest_dir.clone();
                    rdb_dest_path.push(rdb_name);
                    // println!("{}:{}: saving rdb to  {:?}", file!(), line!(), &rdb_dest_path);
                    rdb.save(&rdb_dest_path)?;
                }
            } else {
                eprintln!("ERROR: RDB not found for {}", rdb_name);
            }
        }

        Ok(())
    }

    pub fn update_aoc_hashes_from_modpath<P: AsRef<Path>>(
        &mut self,
        mod_path: P,
    ) -> io::Result<()> {
        // if self.is_valid_mod_dir(&mod_path) {
        let mut p = PathBuf::from(mod_path.as_ref());
        p.push("romfs/asset/data");
        // let files: Vec<AocHash> = fs::read_dir(&p)?
        //     .filter_map(|x| x.ok())
        //     .filter(|x| x.path().is_file())
        //     .map(|x| AocHash::new(x.path(), self.config.clone()))
        //     .filter(|x| x.is_valid())
        //     .collect();

        // self.aoc_hashes.extend(files);
        // }
        if let Ok(entries) = fs::read_dir(&p) {
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
                        }
                    }
                }
            }
        }

        Ok(())
    }

    pub fn get_mods_dirs(&mut self) -> io::Result<()> {
        for entry in fs::read_dir(&self.root_dir)? {
            let path = Pathlib::new(entry?.path());
            if path.is_dir()
                && path.name != self.root_mod_name
                && self.is_valid_mod_dir(&path.full_path)
            {
                self.mods_dirs.push(PathBuf::from(&path.full_path));
            }
        }

        Ok(())
    }

    pub fn get_rdb_name(&self, hash: &AocHash) -> Option<String> {
        for (rdb_name, hashes) in self.config.hashes.iter() {
            if hashes.contains(&hash.hash) {
                return Some(rdb_name.to_string());
            }
        }
        None
    }

    pub fn is_valid_mod_dir<P: AsRef<Path>>(&self, path: P) -> bool {
        let mut p = PathBuf::from(path.as_ref());
        p.push("romfs/asset/data");
        p.exists() && p.is_dir() && std::fs::read_dir(p).is_ok()
    }

    pub fn prepare_master_dir(&mut self) -> io::Result<PathBuf> {
        let mut p = PathBuf::from(&self.root_dir);
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
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Invalid hash"))
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

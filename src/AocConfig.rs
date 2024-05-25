#![allow(non_snake_case, non_camel_case_types)]
use std::{
    collections::HashMap,
    env, fs, io,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::rdb::Rdb;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct AocConfig {
    pub romfs: String,
    #[serde(skip)]
    pub hashes: HashMap<String, Vec<String>>,
    #[serde(skip)]
    pub hashes_rev: HashMap<String, String>,
    #[serde(skip)]
    pub config_path: String,
    #[serde(skip)]
    pub hashes_json_path: String,
}

impl Default for AocConfig {
    fn default() -> Self {
        Self {
            romfs: String::new(),
            hashes: HashMap::new(),
            hashes_rev: HashMap::new(),
            config_path: String::new(),
            hashes_json_path: String::new(),
        }
    }
}

impl AocConfig {
    pub fn safe_new() -> io::Result<AocConfig> {
        match Self::new() {
            Ok(conf) => Ok(conf),
            Err(err) => {
                rfd::MessageDialog::new()
                    .set_buttons(rfd::MessageButtons::Ok)
                    .set_title("Error")
                    .set_description(&format!("{}", err))
                    .show();
                Err(err)
            }
        }
    }
    #[allow(dead_code)]
    pub fn to_json(&self) -> io::Result<serde_json::Value> {
        Ok(json!({
            "romfs": self.romfs,
        }))
    }

    pub fn to_react_json(&self) -> io::Result<serde_json::Value> {
        Ok(json!({
            "romfs": self.romfs,
        }))
    }

    pub fn new() -> io::Result<AocConfig> {
        let mut conf = Self::default();
        conf.get_config_path()?;

        let mut err_str = String::new();

        if let Err(err) = conf.update_default() {
            log_error(&mut err_str, err);
        }

        if conf.try_save_config()? {
            return Ok(conf);
        }

        if let Err(err) = conf.update_from_input() {
            log_error(&mut err_str, err);
        }

        if conf.try_save_config()? {
            return Ok(conf);
        }

        println!("{}", err_str);

        if conf.romfs.is_empty() {
            let e = format!("Unable to get proper romfs path:\n{}", err_str);
            return Err(io::Error::new(io::ErrorKind::NotFound, e));
        }

        conf.save().map_err(|e| {
            io::Error::new(
                io::ErrorKind::NotFound,
                format!("Unable to save config to:\n{}\n{:?}", &conf.config_path, e),
            )
        })?;
        Ok(conf)
    }
    pub fn get_config_path(&mut self) -> io::Result<()> {
        let appdata = env::var("LOCALAPPDATA")
            .map_err(|_| io::Error::new(io::ErrorKind::NotFound, "Cannot access appdata"))?;
        let mut conf_path = PathBuf::from(&appdata);
        conf_path.push("AgeOfCalamity/config.toml");
        makedirs(&conf_path)?;
        self.config_path = conf_path.to_string_lossy().to_string().replace("\\", "/");
        // println!("config_path {:?}", &self.config_path);

        Ok(())
    }

    fn check_if_romfs_valid(romfs: &str) -> bool {
        let mut dest_path = PathBuf::from(romfs);
        dest_path.push("asset/CharacterEditor.rdb"); //example rdb
        dest_path.exists()
    }

    pub fn update_default(&mut self) -> io::Result<()> {
        let conf_str = fs::read_to_string(&self.config_path)?;
        let conf: HashMap<String, serde_json::Value> = toml::from_str(&conf_str).map_err(|e| {
            io::Error::new(
                io::ErrorKind::NotFound,
                format!("Unable to parse default config\n{:?}", e),
            )
        })?;
        let binding = "".into();
        let romfs = conf
            .get("romfs")
            .unwrap_or(&binding)
            .as_str()
            .unwrap_or_default();

        if Self::check_if_romfs_valid(romfs) {
            self.romfs = romfs.to_string().replace("\\", "/");
            return Ok(());
        }

        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "Unable to parse default config",
        ));
    }

    pub fn update_from_input(&mut self) -> io::Result<()> {
        let chosen = rfd::FileDialog::new()
            .set_title("Choose Age of Calamity romfs path")
            .pick_folder()
            .unwrap_or_default();
        let res = chosen.to_string_lossy().to_string().replace("\\", "/");
        if !Self::check_if_romfs_valid(&res) {
            let e = "Invalid romfs path! CharacterEditor.rdb not found:\n";
            rfd::MessageDialog::new()
                .set_buttons(rfd::MessageButtons::Ok)
                .set_title("Invalid romfs path")
                .set_description(e)
                .show();
            return Err(io::Error::new(io::ErrorKind::NotFound, e));
        }
        self.romfs = res;
        Ok(())
    }

    pub fn get_path(&self, pack_local_path: &str) -> Option<PathBuf> {
        //let pack_local_path = format!("Pack/Actor/{}.pack.zs", name);
        let romfs = PathBuf::from(&self.romfs);
        let mut dest_path = romfs.clone();
        dest_path.push(pack_local_path);
        if dest_path.exists() {
            return Some(dest_path);
        }
        None
    }

    pub fn get_rdb_path(&self, name: &str) -> Option<PathBuf> {
        if name.to_lowercase().ends_with(".rdb") {
            return self.get_path(&format!("asset/{}", name));
        }
        self.get_path(&format!("asset/{}.rdb", name))
    }

    pub fn get_rev_hashes(&mut self) {
        if self.hashes_rev.is_empty() {
            for (rdb_name, hashes) in &self.hashes {
                for hash in hashes {
                    self.hashes_rev.insert(hash.clone(), rdb_name.clone());
                }
            }
        }
    }

    pub fn get_hashes(&mut self, force_rebuild: bool) -> io::Result<()> {
        let mut json_path = PathBuf::from(&self.config_path);
        if !json_path.pop() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "Config path has no parent",
            ));
        }
        json_path.push("AOC_hashes.json");
        if json_path.exists() {
            if force_rebuild {
                std::fs::remove_file(&json_path)?;
            } else {
                let json_str = fs::read_to_string(&json_path)?;
                self.hashes = serde_json::from_str(&json_str)?;
                self.get_rev_hashes();
                return Ok(());
            }
        }
        self.hashes_json_path = json_path.to_string_lossy().to_string().replace("\\", "/");
        println!("Generating cache for AOC hashes, this will be done only once...");
        // let mut data: HashMap<String, Vec<String>> = HashMap::new();
        let mut rdb_path = PathBuf::from(&self.romfs);
        rdb_path.push("asset");
        for file in std::fs::read_dir(&rdb_path)? {
            let p = Pathlib::new(file?.path());
            if p.is_file() && p.full_path.to_lowercase().ends_with(".rdb") {
                let mut Hashes: Vec<String> = Vec::new();
                let rdb = Rdb::open(&p.full_path).expect("Failed to open RDB file");
                for entry in &rdb.entries {
                    let hash_formatted = format!("{:08x}", entry.file_ktid);
                    Hashes.push(hash_formatted.clone());
                    self.hashes_rev.insert(hash_formatted, p.name.clone());
                }
                self.hashes.insert(p.name.into(), Hashes);
            }
        }
        serde_json::to_writer(std::fs::File::create(json_path)?, &self.hashes)?;
        Ok(())
    }

    pub fn save(&self) -> io::Result<()> {
        if self.config_path.is_empty() {
            return Err(io::Error::new(io::ErrorKind::NotFound, "Empty config path"));
        }
        makedirs(&PathBuf::from(&self.config_path))?;
        // let json_str: String = serde_json::to_string_pretty(self)?;
        let json_data = self.to_json()?;
        let toml_str = toml::to_string_pretty(&json_data)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, format!("{:#?}", e)))?;
        // write_string_to_file(&self.config_path, &json_str)?;
        let mut res = String::new();
        res.push_str("# Age of Calamity rdb tool merging configuration file\n");
        res.push_str("# \n");
        res.push_str(&toml_str);
        std::fs::write(&self.config_path, &res)?;
        Ok(())
    }

    fn try_save_config(&mut self) -> io::Result<bool> {
        if !self.romfs.is_empty() {
            self.save().map_err(|e| {
                io::Error::new(
                    io::ErrorKind::NotFound,
                    format!("Unable to save config to:\n{}\n{:?}", &self.config_path, e),
                )
            })?;
            self.get_hashes(false)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

pub fn makedirs<P: AsRef<Path>>(path: P) -> std::io::Result<()> {
    let binding = path.as_ref();
    let par = Path::new(&binding).parent();
    if let Some(par) = par {
        fs::create_dir_all(par)?;
    }
    Ok(())
}

fn log_error(err_str: &mut String, err: io::Error) {
    let e = format!("{:#?}\n", err);
    println!("{}", &e);
    err_str.push_str(&e);
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Pathlib {
    pub parent: String,
    pub name: String,
    pub stem: String,
    pub extension: String,
    pub ext_last: String,
    pub full_path: String,
}

impl Default for Pathlib {
    fn default() -> Self {
        Self::new("")
    }
}

impl Pathlib {
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        let path_str = path.as_ref().to_str().unwrap_or_default().to_string();
        Self {
            parent: Pathlib::get_parent(&path),
            name: Pathlib::get_name(&path),
            stem: Pathlib::get_stem(&path),
            extension: Pathlib::get_extension(&path),
            ext_last: Self::get_ext_last(&path),
            full_path: path_str,
        }
    }

    pub fn is_file(&self) -> bool {
        Path::new(&self.full_path).is_file()
    }

    pub fn is_dir(&self) -> bool {
        Path::new(&self.full_path).is_dir()
    }

    pub fn get_ext_last<P: AsRef<Path>>(path: P) -> String {
        let extension = Pathlib::get_extension(&path);
        if !extension.contains('.') {
            return "".to_string();
        }
        extension.split('.').last().unwrap_or_default().to_string()
    }

    pub fn get_parent<P: AsRef<Path>>(path: P) -> String {
        Path::new(path.as_ref())
            .parent()
            .and_then(|p| p.to_str())
            .map(|s| s.to_string())
            .unwrap_or_default()
    }

    pub fn get_name<P: AsRef<Path>>(path: P) -> String {
        Path::new(path.as_ref())
            .file_name()
            .and_then(|p| p.to_str())
            .map(|s| s.to_string())
            .unwrap_or_default()
    }

    pub fn get_stem<P: AsRef<Path>>(path: P) -> String {
        let res = Path::new(path.as_ref())
            .file_stem()
            .and_then(|p| p.to_str())
            .map(|s| s.to_string())
            .unwrap_or_default();
        if res.contains('.') {
            return res.split('.').next().unwrap_or_default().to_string();
        }
        res
    }

    pub fn get_extension<P: AsRef<Path>>(path: P) -> String {
        let path_str = path.as_ref().to_str().unwrap_or_default();
        let dots = path_str.chars().filter(|&x| x == '.').count();
        if dots == 0 {
            return String::new();
        }
        if dots > 1 {
            return path_str.split('.').skip(1).collect::<Vec<&str>>().join(".");
        }
        Path::new(path.as_ref())
            .extension()
            .and_then(|p| p.to_str())
            .map(|s| s.to_string())
            .unwrap_or_default()
    }
}


pub fn normalize_path(path: PathBuf) -> PathBuf {
    let prefix = r"\\?\";
    if path.to_str().map_or(false, |s| s.starts_with(prefix)) {
        PathBuf::from(path.to_str().unwrap().trim_start_matches(prefix))
    } else {
        path
    }
}
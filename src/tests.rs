#![allow(dead_code)]
use std::{io, path::PathBuf};
use binread::{io::Cursor, BinRead};
use binwrite::BinWrite;
mod rdb;
use rdb::Rdb;
mod ktid;
use ktid::ktid;
mod typeinfo;
use structopt::StructOpt;


mod tests {
    use std::io::Write;

    use super::*;

    //const TEST_CONTENTS: &[u8] = include_bytes!("../system.rdb");

    #[test]
    fn test() {
        let test = typeinfo::object::sound::bank::ID;
    }

    use serde::Deserialize;

    #[derive(Debug, Deserialize)]
    pub struct TypeInfoEntry {
        pub typekind: String,
        ktid: String,
        pub typename: String,
    }

    #[test]
    fn generate_typeinfos_lmao_gross() {
        let mut rdr = csv::ReaderBuilder::new()
            .has_headers(false)
            .from_path("typeinfos.csv")
            .unwrap();

        let typeinfos: Vec<String> = rdr
            .deserialize()
            .into_iter()
            .filter_map(|result| {
                let record: TypeInfoEntry = result.unwrap();

                //dbg!(record);
                if record.typekind == "TypeInfo" {
                    Some(record.typename)
                } else {
                    None
                }
            })
            .collect();

        for typeinfo in &typeinfos {
            let mut path =
                PathBuf::from(format!(".\\src\\{}", &typeinfo.replace("::", "\\")).to_lowercase());
            std::fs::create_dir_all(&path).unwrap();

            let mut mod_path = path.join("mod.rs");

            if !mod_path.exists() {
                let mut file = std::fs::OpenOptions::new()
                    .create(true)
                    .write(true)
                    .open(&dbg!(&mod_path))
                    .unwrap();
                let stem = mod_path
                    .parent()
                    .unwrap()
                    .file_stem()
                    .unwrap()
                    .to_str()
                    .unwrap();
                file.write_all(
                    format!(
                        "use crate::ktid::KTID;\n\npub const ID: KTID = KTID({});",
                        crate::ktid::ktid(&typeinfo).as_u32()
                    )
                    .as_bytes(),
                )
                .unwrap();
            }
        }

        for typeinfo in &typeinfos {
            let mut path = PathBuf::from(format!(".\\src\\{}", &typeinfo.replace("::", "\\")));

            path.ancestors().for_each(|ancestor| {
                let dirs: Vec<String> = std::fs::read_dir(ancestor)
                    .unwrap()
                    .filter_map(|dir| {
                        let dir = dir.unwrap();
                        if dir.path().is_dir() {
                            Some(dir.file_name().to_str().unwrap().to_string())
                        } else {
                            None
                        }
                    })
                    .collect();

                let mod_path = ancestor.join("mod.rs");

                if !mod_path.exists() {
                    let mut output = String::new();

                    for dir in dirs {
                        output.push_str(&format!("pub mod {};\n", dir));
                    }

                    std::fs::write(&mod_path, &output);
                }
            });
        }

        println!(
            "{:x}",
            ktid::ktid("TypeInfo::Object::3D::Displayset::TrianglesEx").as_u32()
        )
    }

    // #[test]
    // fn type_8_search() {
    //     let mut rdb: Rdb = Rdb::read(&mut Cursor::new(TEST_CONTENTS)).unwrap();
    //     //let entry = rdb.get_entry_by_KTID(0xf82a2296).unwrap();
    //     let entry = rdb.entries.iter().find(|lmao| lmao.entry_type != 0 && lmao.entry_type != 1 && lmao.entry_type != 4 && lmao.entry_type != 8 && lmao.entry_type != 12 && lmao.string_size != 0);
    //     dbg!(entry);
    // }

    #[test]
    fn patch_texternal() {
        //let mut rdb: Rdb = Rdb::read(&mut Cursor::new(TEST_CONTENTS)).unwrap();
        // patch_rdb(&Opt { path: PathBuf::from("RRPreview.rdb"), out_path: PathBuf::from("RRPreview.rdb"), data_path: PathBuf::from("data") });
        //patch_rdb(Path::new("KIDSSystemResource.rdb"), Path::new("cock.rdb"));
        // let entry = rdb.get_entry_by_KTID(0x0a696242).unwrap();
        // entry.patch_external_file();
        //dbg!(entry);
    }
}

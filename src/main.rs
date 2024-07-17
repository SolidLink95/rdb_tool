#![allow(dead_code)]
#![allow(non_snake_case, non_camel_case_types)]
use binread::{io::Cursor, BinRead};
use binwrite::BinWrite;
use utils::copy_dirs;
use ModMerger::AocHash;
mod ModMerger;
mod utils;
use std::{env, io::{self, Read, Stdin}, os::windows::process, path::PathBuf, sync::Arc};
mod AocConfig;
mod rdb;
use rdb::Rdb;
mod ktid;
use ktid::ktid;
mod typeinfo;
use structopt::StructOpt;

use crate::AocConfig::normalize_path;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "RdbTool",
    about = "Simple command-line tool to manipulate RDB files."
)]
struct Opt {
    #[structopt(subcommand)]
    cmd: Command,
}

#[derive(Debug, StructOpt)]
enum Command {
    /// Patch a RDB file
    Patch(Patch),
    /// Output relevant informations about a RDB entry
    Print(Print),
}

#[derive(Debug, StructOpt)]
struct Patch {
    #[structopt(parse(from_os_str), help = "Path to the RDB file")]
    pub path: PathBuf,
    #[structopt(parse(from_os_str), help = "Output path to the RDB file")]
    pub out_path: PathBuf,
    #[structopt(
        parse(from_os_str),
        default_value = "patch",
        help = "Directory where the files to patch are located"
    )]
    pub data_path: PathBuf,
}

#[derive(Debug, StructOpt)]
struct Print {
    #[structopt(parse(from_os_str), help = "Path to the RDB file")]
    pub path: PathBuf,
    #[structopt(help = "The KTID you would like to print")]
    pub ktid: String,
}

fn patch_rdb(args: &Patch) -> io::Result<()> {
    let config = Arc::new(AocConfig::AocConfig::safe_new()?);
    let mut rdb =
        Rdb::open(&args.path).expect(&format!("Failed to open RDB file: {:?}", args.path));

    let external_path = if args.data_path.is_relative() {
        let rdb_dir = if args.path.is_relative() {
            std::fs::canonicalize(&args.path)
                .expect("Failed to canonicalize path")
                .parent()
                .expect("Failed to get parent directory for RDB file")
                .to_path_buf()
        } else {
            args.path
                .parent()
                .expect("Unable to get parent dir for RDB file")
                .to_path_buf()
        };

        rdb_dir.join(&args.data_path)
    } else {
        args.data_path.to_path_buf()
    };

    let external_path = normalize_path(external_path);
    println!("{}: External path: {}", line!(), external_path.display());

    if !external_path.exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!(
                "Couldn't find a directory to patch ('{}' was used). Consider making it?",
                external_path.display()
            ),
        ));
    }

    let files = match std::fs::read_dir(external_path) {
        Ok(files) => files,
        Err(e) => {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("Couldn't read the directory: {}", e),
            ));
        }
    };

    for entry in files {
        let entry = entry?;
        println!("{}: entry {:?}", line!(), entry);
        let metadata = entry.metadata()?;

        // We don't care about subdirectories
        if metadata.is_dir() {
            continue;
        }

        let path = &entry.path();

        // Check if we're dealing with a KTID or an actual filename
        let filename = if path
            .file_name()
            .and_then(|x| x.to_str())
            .expect(&format!("Invalid file_name: {}", path.display()))
            .to_lowercase()
            .starts_with("0x")
        {
            // Strip the extension (Cethleann keeps the extension even if the hash is missing)
            path.file_stem()
                .and_then(|x| x.to_str())
                .expect(&format!("Invalid file_stem: {}", path.display()))
        } else {
            // Get the full filename with extension
            path.file_name()
                .and_then(|x| x.to_str())
                .expect(&format!("Invalid file_name: {}", path.display()))
        };
        println!("{}: filename {:?}", line!(), &filename);
        println!("{}: entry.path() {:?}", line!(), &entry.path());

        match rdb.get_entry_by_ktid_mut(crate::ktid(filename)) {
            Some(entry_found) => {
                println!("Patching {}", filename);
                entry_found.make_external();
                entry_found.make_uncompressed();
                let aoc_hash = AocHash::new(&entry.path(), config.clone());
                if let Ok(_) = entry_found.set_external_file(&aoc_hash) {
                    println!("{}: set_external_file success", line!());
                } else {
                    println!("{}: set_external_file failed", line!());
                }
                // entry_found
                //     .set_external_file(&entry.path())
                //     .expect("Failed to set external file");
            }
            None => println!("File {} not found in the RDB. Skipping.", filename),
        }
    }

    let mut bytes = vec![];
    rdb.write(&mut bytes)?;

    std::fs::write(&args.out_path, bytes)
}

fn main() -> io::Result<()> {
    let matches = clap::Command::new("AOC mods merger")
        .version("1.0")
        .about("Merging mods for Age of Calamity")
        .arg(
            clap::Arg::new("job_path")
                .help("Path to directory containing all mods")
                .required(false)
                .index(1),
        )
        .arg(
            clap::Arg::new("output")
                .short('o')
                .long("output")
                .help("Optional argument where to copy the merged mod directory")
                .required(false)
        )
        .get_matches();

    // Optional job path
    let tmp = "".to_string();
    let cwd_dir = matches.get_one::<String>("job_path").unwrap_or_else(|| &tmp).to_string();
    let output_dir = matches.get_one::<String>("output").unwrap_or_else(|| &tmp).to_string();
    // let argv = env::args().nth(1).unwrap_or_default(); 
    let working_dir = if !cwd_dir.is_empty() {
        PathBuf::from(cwd_dir)
    } else {
        env::current_dir()?
    };
    let emulator_name = utils::is_emulator_dir(&working_dir);
    if !emulator_name.is_empty() {
        println!("It seems You are trying to work directly inside {} emulator mod directory.", emulator_name);
        println!("Please change the working directory in order to avoid permanent damage to save game files.");
        println!("Press any key to exit...");
        if let Ok(_) = io::stdin().read(&mut [0u8]) {
            return Ok(());
        } 
        std::process::exit(1);
    } 


    // let p = r"C:\Users\Mati\AppData\Roaming\yuzu\load\01002B00111A2000";
    let mut modmerger = ModMerger::ModMerger::new::<PathBuf>(Some(working_dir))?;
    if let Err(e) = modmerger.process_mods() {
        println!("Error: {}", e);

    } else {
        println!("Done processing mods");
        if !output_dir.is_empty() {
            copy_dirs(&modmerger.root_dir.path, &PathBuf::from(output_dir))?;
        }
    }


    // return Ok(());
    // AocConfig::AocConfig::safe_new();
    // match Opt::from_args_safe() {
    //     Ok(opt) => match opt.cmd {
    //         Command::Patch(args) => {
    //             if let Err(error_msg) = patch_rdb(&args) {
    //                 println!("{}", error_msg);
    //             }
    //         }
    //         Command::Print(args) => {
    //             let ktid = ktid(&args.ktid);
    //             let rdb = Rdb::read(&mut Cursor::new(&std::fs::read(&args.path)?))
    //                 .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    //             if let Some(entry) = rdb.get_entry_by_ktid(&ktid) {
    //                 println!("{:#?}", entry);
    //             } else {
    //                 println!("KTID {:?} not found in the RDB.", &ktid);
    //             }
    //         }
    //     },
    //     Err(e) => {
    //         println!("{}", e.message);
    //     }
    // }

    Ok(())
}

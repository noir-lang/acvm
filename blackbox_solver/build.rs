use std::{
    fs::File,
    path::{Path, PathBuf},
};

use flate2::read::GzDecoder;
use tar::Archive;

const BARRETENBERG_ARCHIVE: &&str = &"BARRETENBERG_ARCHIVE";
const BARRETENBERG_BIN_DIR: &&str = &"BARRETENBERG_BIN_DIR";

fn unpack_wasm(archive_path: &Path, target_dir: &Path) -> Result<(), String> {
    if archive_path.exists() && archive_path.is_file() {
        let archive = File::open(archive_path).map_err(|_| "Could not read archive")?;
        let gz_decoder: GzDecoder<File> = GzDecoder::new(archive);
        let mut archive = Archive::new(gz_decoder);

        archive.unpack(target_dir).unwrap();

        Ok(())
    } else {
        Err(format!("Unable to locate {BARRETENBERG_ARCHIVE} - Please set the BARRETENBERG_BIN_DIR env var to the directory where it exists, or ensure it's located at {}", archive_path.display()))
    }
}

fn main() -> Result<(), String> {
    let out_dir = std::env::var("OUT_DIR").unwrap();

    match std::env::var(BARRETENBERG_ARCHIVE) {
        Ok(archive_path) => {
            unpack_wasm(&PathBuf::from(archive_path), &PathBuf::from(&out_dir))?;
            println!("cargo:rustc-env={BARRETENBERG_BIN_DIR}={out_dir}");
            Ok(())
        }
        Err(error) => Err(error.to_string()),
    }
}

const BLACK_BOX_SOLVER_BIN_DIR: &&str = &"BLACK_BOX_SOLVER_BIN_DIR";
const BB_WASM: &&str = &"barretenberg.wasm";

fn main() -> Result<(), String> {
    match std::env::var(BLACK_BOX_SOLVER_BIN_DIR) {
        Ok(bindir) => {
            println!("cargo:rustc-env={BLACK_BOX_SOLVER_BIN_DIR}={bindir}");
            Ok(())
        }
        Err(_) => {
            if let Ok(bindir) = pkg_config::get_variable("barretenberg", "bindir") {
                println!("cargo:rustc-env={BLACK_BOX_SOLVER_BIN_DIR}={bindir}");
                Ok(())
            } else {
                let current_dir = std::env::current_dir().unwrap();
                let bin_path = current_dir.join("result").join("bin");
                let wasm_path = bin_path.join(BB_WASM);
                let bin_path_string = bin_path.to_string_lossy();
                if wasm_path.exists() && wasm_path.is_file() {
                    println!("cargo:warning=BLACK_BOX_SOLVER_BIN_DIR env not set, setting BLACK_BOX_SOLVER_BIN_DIR={bin_path_string}");
                    println!("cargo:rustc-env={BLACK_BOX_SOLVER_BIN_DIR}={bin_path_string}");
                    Ok(())
                } else {
                    Err(format!("Unable to locate barretenberg.wasm - Please set the BLACK_BOX_SOLVER_BIN_DIR env var to the directory where it exists, or ensure it's located at {bin_path_string}", ))
                }
            }
        }
    }
}

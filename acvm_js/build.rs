const GIT_COMMIT: &&str = &"GIT_COMMIT";
const BARRETENBERG_BIN_DIR: &&str = &"BARRETENBERG_BIN_DIR";

fn main() -> Result<(), String> {
    if std::env::var(GIT_COMMIT).is_err() {
        build_data::set_GIT_COMMIT();
        build_data::set_GIT_DIRTY();
        build_data::no_debug_rebuilds();
    }

    match std::env::var(BARRETENBERG_BIN_DIR) {
        Ok(bindir) => {
            println!("cargo:rustc-env={BARRETENBERG_BIN_DIR}={bindir}");
            Ok(())
        }
        Err(_) => {
            if let Ok(bindir) = pkg_config::get_variable("barretenberg", "bindir") {
                println!("cargo:rustc-env={BARRETENBERG_BIN_DIR}={bindir}");
                Ok(())
            } else {
                Err("Unable to locate barretenberg.wasm - Please set the BARRETENBERG_BIN_DIR env var to the directory where it exists".into())
            }
        }
    }
}

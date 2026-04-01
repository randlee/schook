use atm_extension::AtmExtensionHandler;
use sc_hooks_sdk::runner::PluginRunner;

fn main() {
    std::process::exit(PluginRunner::run_sync(&AtmExtensionHandler));
}

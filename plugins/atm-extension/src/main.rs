use atm_extension::AtmExtensionHandler;
use mimalloc::MiMalloc;
use sc_hooks_sdk::runner::PluginRunner;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

fn main() {
    std::process::exit(PluginRunner::run_sync(&AtmExtensionHandler));
}

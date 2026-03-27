use atm_bash_identity::AtmBashIdentity;
use sc_hooks_sdk::runner::PluginRunner;

fn main() {
    let handler = AtmBashIdentity;
    std::process::exit(PluginRunner::run_sync(&handler));
}

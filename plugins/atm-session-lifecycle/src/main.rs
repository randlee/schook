use atm_session_lifecycle::AtmSessionLifecycle;
use sc_hooks_sdk::runner::PluginRunner;

fn main() {
    let handler = AtmSessionLifecycle;
    std::process::exit(PluginRunner::run_sync(&handler));
}

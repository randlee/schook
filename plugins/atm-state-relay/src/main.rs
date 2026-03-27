use atm_state_relay::AtmStateRelay;
use sc_hooks_sdk::runner::PluginRunner;

fn main() {
    let handler = AtmStateRelay;
    std::process::exit(PluginRunner::run_sync(&handler));
}

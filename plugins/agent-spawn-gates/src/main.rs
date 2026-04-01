use agent_spawn_gates::AgentSpawnGatesHandler;
use sc_hooks_sdk::runner::PluginRunner;

fn main() {
    std::process::exit(PluginRunner::run_sync(&AgentSpawnGatesHandler));
}

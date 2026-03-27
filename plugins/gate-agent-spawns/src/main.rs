use gate_agent_spawns::GateAgentSpawns;
use sc_hooks_sdk::runner::PluginRunner;

fn main() {
    let handler = GateAgentSpawns;
    std::process::exit(PluginRunner::run_sync(&handler));
}

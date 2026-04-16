use agent_spawn_gates::AgentSpawnGatesHandler;
use mimalloc::MiMalloc;
use sc_hooks_sdk::runner::PluginRunner;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

fn main() {
    std::process::exit(PluginRunner::run_sync(&AgentSpawnGatesHandler));
}

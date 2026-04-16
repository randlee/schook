use mimalloc::MiMalloc;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

fn main() {
    let handler = agent_session_foundation::SessionFoundationHandler;
    std::process::exit(sc_hooks_sdk::runner::PluginRunner::run_sync(&handler));
}

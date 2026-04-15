use mimalloc::MiMalloc;
use sc_hooks_sdk::runner::PluginRunner;
use tool_output_gates::ToolOutputGatesHandler;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

fn main() {
    std::process::exit(PluginRunner::run_sync(&ToolOutputGatesHandler));
}

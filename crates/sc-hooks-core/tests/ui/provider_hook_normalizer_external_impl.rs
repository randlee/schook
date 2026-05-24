use sc_hooks_core::normalization::{
    NormalizedHookContext, ProviderHookInput, ProviderHookNormalizer, ProviderHookSource,
};

struct ForeignNormalizer;

impl ProviderHookNormalizer for ForeignNormalizer {
    fn normalize<'a>(
        &self,
        raw: ProviderHookInput<'a>,
    ) -> Result<NormalizedHookContext<'a>, sc_hooks_core::errors::HookError> {
        let _ = ProviderHookSource::Codex;
        let _ = raw;
        unimplemented!()
    }
}

fn main() {}

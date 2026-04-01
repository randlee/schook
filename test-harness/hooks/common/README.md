# Common Harness Components

This directory is reserved for shared harness logic and shared harness policy.

Expected future contents:

- shared report-generation helpers
- shared redaction helpers
- shared fixture utilities
- shared schema-drift comparison logic
- shared `pytest` helpers or fixtures

Provider-specific assumptions should not be stored here.

If a helper depends on a provider-specific payload shape, it belongs in that
provider's directory instead.

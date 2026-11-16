#[derive(Debug, Clone, Copy)]
pub struct WalletCapabilityDefinition {
    pub label: &'static str,
    pub description: &'static str,
    pub externally_observable: bool,
    pub accesses_sensitive_secret: bool,
}

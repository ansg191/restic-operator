#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Any error originating from the `kube-rs` crate
    #[error("Kubernetes reported error: {0}")]
    KubeError(#[from] kube::Error),
    /// Error in serializing the resticprofile config to TOML
    #[error("Error creating resticprofile config: {0}")]
    TomlSerializeError(#[from] toml::ser::Error),
    /// Missing Namespace
    #[error("Namespace not found")]
    MissingNamespace,
}

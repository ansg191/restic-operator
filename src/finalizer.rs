use kube::{
    api::{Patch, PatchParams},
    Api, Error,
};
use serde::de::DeserializeOwned;
use serde_json::{json, Value};

pub const FINALIZER: &str = "restic.anshulg.com/finalizer";

/// Adds a finalizer to the given resource.
pub async fn add<K>(api: &Api<K>, name: &str) -> Result<K, Error>
where
    K: Clone + DeserializeOwned + std::fmt::Debug,
{
    let finalizer: Value = json!({
        "metadata": {
            "finalizers": [FINALIZER]
        }
    });

    let patch = Patch::Merge(&finalizer);
    api.patch(name, &PatchParams::default(), &patch).await
}

/// Removes all finalizers from the given resource.
pub async fn remove<K>(api: &Api<K>, name: &str) -> Result<K, Error>
where
    K: Clone + DeserializeOwned + std::fmt::Debug,
{
    let finalizer: Value = json!({
        "metadata": {
            "finalizers": null
        }
    });

    let patch = Patch::Merge(&finalizer);
    api.patch(name, &PatchParams::default(), &patch).await
}

#[cfg(test)]
mod tests {
    use k8s_openapi::api::core::v1::ConfigMap;
    use kube::{api::ObjectMeta, Client, ResourceExt};

    use super::*;

    #[tokio::test]
    #[cfg_attr(
        not(feature = "integration-tests"),
        ignore = "uses k8s current-context"
    )]
    async fn test_finalizer() {
        let client = Client::try_default().await.unwrap();
        let api: Api<ConfigMap> = Api::namespaced(client, "default");

        // Create a test ConfigMap
        let cm = ConfigMap {
            metadata: ObjectMeta {
                name: Some("test-finalizer-add".to_string()),
                ..Default::default()
            },
            ..Default::default()
        };
        let cm = api.create(&Default::default(), &cm).await.unwrap();

        // Check for no finalizers
        assert!(cm.metadata.finalizers.is_none());

        // Add the finalizer
        let cm = add(&api, &cm.name_any()).await.unwrap();

        // Check for the finalizer
        assert_eq!(cm.metadata.finalizers, Some(vec![FINALIZER.to_string()]));

        // Remove the finalizer
        let cm = remove(&api, &cm.name_any()).await.unwrap();

        // Check for no finalizers
        assert!(cm.metadata.finalizers.is_none());

        // Clean up
        api.delete(&cm.name_any(), &Default::default())
            .await
            .unwrap();
    }
}

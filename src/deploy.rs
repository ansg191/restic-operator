use std::{collections::BTreeMap, future::Future};

use kube::{Client, Resource};

pub trait Deployable {
    type Error;

    fn create<O>(
        &self,
        client: Client,
        owner: &O,
        labels: Labels,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send
    where
        O: Resource<DynamicType = ()> + Send + Sync;
    fn delete(&self, client: Client) -> impl Future<Output = Result<(), Self::Error>> + Send;
}

#[derive(Debug, Clone)]
pub struct Labels {
    app_name: String,
}

impl Labels {
    pub fn new(app_name: impl Into<String>) -> Self {
        Self {
            app_name: app_name.into(),
        }
    }

    pub fn to_labels(&self) -> BTreeMap<String, String> {
        let mut labels = BTreeMap::new();
        labels.insert(
            "app.kubernetes.io/name".to_owned(),
            self.app_name.to_owned(),
        );
        labels.insert(
            "app.kubernetes.io/managed-by".to_owned(),
            "restic-operator".to_owned(),
        );
        labels
    }
}

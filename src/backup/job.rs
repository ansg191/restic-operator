use k8s_openapi::{api::batch::v1::Job, apimachinery::pkg::apis::meta::v1::OwnerReference};
use kube::{api::ObjectMeta, Api, Resource, ResourceExt};
use restic_crd::Backup;

use crate::{deploy::Deployable, jobspec::BackupJobSpec, Error};

#[derive(Debug, Clone)]
pub struct BackupJob {
    name: String,
    ns: String,
    spec: BackupJobSpec,
}

impl BackupJob {
    pub fn new(ns: impl Into<String>, backup: &Backup, config_name: impl Into<String>) -> Self {
        let spec = BackupJobSpec::new(&backup.spec, config_name);
        Self {
            name: format!("{}-job", backup.name_any()),
            ns: ns.into(),
            spec,
        }
    }

    async fn get(&self, client: kube::Client) -> Result<Option<Job>, Error> {
        let api: Api<Job> = Api::namespaced(client, &self.ns);
        match api.get(&self.name).await {
            Ok(c) => Ok(Some(c)),
            Err(kube::Error::Api(ae)) => {
                if ae.code == 404 {
                    Ok(None)
                } else {
                    Err(Error::KubeError(kube::Error::Api(ae)))
                }
            }
            Err(e) => Err(Error::KubeError(e)),
        }
    }
}

impl Deployable for BackupJob {
    type Error = Error;

    async fn create<O>(
        &self,
        client: kube::Client,
        owner: &O,
        labels: crate::deploy::Labels,
    ) -> Result<(), Self::Error>
    where
        O: Resource<DynamicType = ()> + Send + Sync,
    {
        let job = Job {
            metadata: ObjectMeta {
                name: Some(self.name.clone()),
                namespace: Some(self.ns.clone()),
                labels: Some(labels.to_labels()),
                owner_references: O::meta(owner).uid.clone().map(|uid| {
                    vec![OwnerReference {
                        api_version: O::api_version(&()).into_owned(),
                        block_owner_deletion: Some(true),
                        controller: Some(true),
                        kind: O::kind(&()).into_owned(),
                        name: O::name_any(owner),
                        uid,
                    }]
                }),
                ..Default::default()
            },
            spec: Some(self.spec.clone().into()),
            ..Default::default()
        };

        let api: Api<Job> = Api::namespaced(client, &self.ns);
        api.create(&Default::default(), &job).await?;

        Ok(())
    }

    async fn delete(&self, client: kube::Client) -> Result<(), Self::Error> {
        let job = self.get(client.clone()).await?;
        if let Some(job) = job {
            let api: Api<Job> = Api::namespaced(client, &self.ns);
            api.delete(&job.name_any(), &Default::default()).await?;
        }
        Ok(())
    }
}

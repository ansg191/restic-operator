use kube::{Client, ResourceExt};
use restic_crd::Backup;

use super::job::BackupJob;
use crate::{
    deploy::{Deployable, Labels},
    resticprofile::ResticProfile,
    Error,
};

#[derive(Debug, Clone)]
pub struct BackupDeployment {
    profile: ResticProfile,
    job: BackupJob,
}

impl BackupDeployment {
    pub fn new(ns: String, backup: &Backup) -> Self {
        let profile = ResticProfile::new(ns.clone(), backup.name_any(), &backup.spec);
        let job = BackupJob::new(ns, backup, profile.name());
        Self { profile, job }
    }
}

impl Deployable for BackupDeployment {
    type Error = Error;

    async fn create<O>(&self, client: Client, owner: &O, labels: Labels) -> Result<(), Self::Error>
    where
        O: kube::Resource<DynamicType = ()> + Send + Sync,
    {
        self.profile
            .create(client.clone(), owner, labels.clone())
            .await?;
        self.job.create(client, owner, labels).await?;
        Ok(())
    }

    async fn delete(&self, client: Client) -> Result<(), Self::Error> {
        self.profile.delete(client.clone()).await?;
        self.job.delete(client).await?;
        Ok(())
    }
}

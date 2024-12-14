use kube::ResourceExt;
use restic_crd::ScheduledBackup;

use super::{cronjob::BackupCronJob, Error};
use crate::{deploy::Deployable, resticprofile::ResticProfile};

#[derive(Debug, Clone)]
pub struct ScheduledBackupDeployment {
    profile: ResticProfile,
    job: BackupCronJob,
}

impl ScheduledBackupDeployment {
    pub fn new(ns: String, backup: &ScheduledBackup) -> Self {
        let profile = ResticProfile::new(ns.clone(), backup.name_any(), &backup.spec.backup);
        let job = BackupCronJob::new(ns, backup, profile.name());
        Self { profile, job }
    }
}

impl Deployable for ScheduledBackupDeployment {
    type Error = Error;

    async fn create<O>(
        &self,
        client: kube::Client,
        owner: &O,
        labels: crate::deploy::Labels,
    ) -> Result<(), Self::Error>
    where
        O: kube::Resource<DynamicType = ()> + Send + Sync,
    {
        self.profile
            .create(client.clone(), owner, labels.clone())
            .await?;
        self.job.create(client, owner, labels).await?;
        Ok(())
    }

    async fn delete(&self, client: kube::Client) -> Result<(), Self::Error> {
        self.profile.delete(client.clone()).await?;
        self.job.delete(client).await?;
        Ok(())
    }
}

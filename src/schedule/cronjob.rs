use k8s_openapi::{
    api::batch::v1::{CronJob, CronJobSpec, JobTemplateSpec},
    apimachinery::pkg::apis::meta::v1::OwnerReference,
};
use kube::{api::ObjectMeta, Api, Resource, ResourceExt};
use restic_crd::ScheduledBackup;

use super::Error;
use crate::{deploy::Deployable, jobspec::BackupJobSpec};

#[derive(Debug, Clone)]
pub struct BackupCronJob {
    name: String,
    ns: String,
    spec: BackupJobSpec,

    /// The schedule in Cron format, see https://en.wikipedia.org/wiki/Cron.
    pub schedule: String,
    /// Specifies how to treat concurrent executions of a Job. Valid values are:
    ///
    /// - "Allow" (default): allows CronJobs to run concurrently; - "Forbid": forbids concurrent runs, skipping next run if previous run hasn't finished yet; - "Replace": cancels currently running job and replaces it with a new one
    pub concurrency_policy: Option<String>,
    /// The number of failed finished jobs to retain. Value must be non-negative integer. Defaults to 1.
    pub failed_jobs_history_limit: Option<i32>,
    /// Optional deadline in seconds for starting the job if it misses scheduled time for any reason.  Missed jobs executions will be counted as failed ones.
    pub starting_deadline_seconds: Option<i64>,
    /// The number of successful finished jobs to retain. Value must be non-negative integer. Defaults to 3.
    pub successful_jobs_history_limit: Option<i32>,
    /// This flag tells the controller to suspend subsequent executions, it does not apply to already started executions.  Defaults to false.
    pub suspend: Option<bool>,
    /// The time zone name for the given schedule, see https://en.wikipedia.org/wiki/List_of_tz_database_time_zones. If not specified, this will default to the time zone of the kube-controller-manager process. The set of valid time zone names and the time zone offset is loaded from the system-wide time zone database by the API server during CronJob validation and the controller manager during execution. If no system-wide time zone database can be found a bundled version of the database is used instead. If the time zone name becomes invalid during the lifetime of a CronJob or due to a change in host configuration, the controller will stop creating new new Jobs and will create a system event with the reason UnknownTimeZone. More information can be found in https://kubernetes.io/docs/concepts/workloads/controllers/cron-jobs/#time-zones
    pub time_zone: Option<String>,
}

impl BackupCronJob {
    pub fn new(
        ns: impl Into<String>,
        backup: &ScheduledBackup,
        config_name: impl Into<String>,
    ) -> Self {
        let spec = BackupJobSpec::new(&backup.spec.backup, config_name);
        Self {
            name: format!("{}-cronjob", backup.name_any()),
            ns: ns.into(),
            spec,
            schedule: backup.spec.schedule.clone(),
            concurrency_policy: backup.spec.concurrency_policy.clone(),
            failed_jobs_history_limit: backup.spec.failed_jobs_history_limit,
            starting_deadline_seconds: backup.spec.starting_deadline_seconds,
            successful_jobs_history_limit: backup.spec.successful_jobs_history_limit,
            suspend: backup.spec.suspend,
            time_zone: backup.spec.time_zone.clone(),
        }
    }

    async fn get(&self, client: kube::Client) -> Result<Option<CronJob>, Error> {
        let api: Api<CronJob> = Api::namespaced(client, &self.ns);
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

impl Deployable for BackupCronJob {
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
        let job = CronJob {
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
            spec: Some(CronJobSpec {
                concurrency_policy: self.concurrency_policy.clone(),
                failed_jobs_history_limit: self.failed_jobs_history_limit,
                job_template: JobTemplateSpec {
                    metadata: None,
                    spec: Some(self.spec.clone().into()),
                },
                schedule: self.schedule.clone(),
                starting_deadline_seconds: self.starting_deadline_seconds,
                successful_jobs_history_limit: self.successful_jobs_history_limit,
                suspend: self.suspend,
                time_zone: self.time_zone.clone(),
            }),
            ..Default::default()
        };

        let api: Api<CronJob> = Api::namespaced(client, &self.ns);
        api.create(&Default::default(), &job).await?;

        Ok(())
    }

    async fn delete(&self, client: kube::Client) -> Result<(), Self::Error> {
        let job = self.get(client.clone()).await?;
        if let Some(job) = job {
            let api: Api<CronJob> = Api::namespaced(client, &self.ns);
            api.delete(&job.name_any(), &Default::default()).await?;
        }
        Ok(())
    }
}

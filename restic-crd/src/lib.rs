use bon::Builder;
use k8s_openapi::{
    api::core::v1::{
        Affinity, EnvFromSource, EnvVar, ResourceRequirements, SecretKeySelector, SecurityContext,
        Volume, VolumeMount,
    },
    apimachinery::pkg::apis::meta::v1::Time,
};
use kube::CustomResource;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(CustomResource, Serialize, Deserialize, Debug, PartialEq, Clone, JsonSchema, Builder)]
#[kube(
    group = "restic.anshulg.com",
    version = "v1alpha1",
    kind = "ScheduledBackup",
    plural = "scheduled-backups",
    derive = "PartialEq",
    status = "ScheduledBackupStatus",
    shortname = "rsb",
    category = "restic",
    printcolumn = r#"{"name": "Age", "type": "date", "jsonPath": ".metadata.creationTimestamp"}"#,
    // printcolumn = r#"{"name": "Phase", "type": "string", "jsonPath": ".status.phase"}"#,
    namespaced
)]
#[serde(rename_all = "camelCase")]
pub struct ScheduledBackupSpec {
    /// The schedule in Cron format, see https://en.wikipedia.org/wiki/Cron.
    pub schedule: String,
    /// The backup spec
    pub backup: BackupSpec,
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

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema, Builder)]
#[serde(rename_all = "camelCase")]
pub struct ScheduledBackupStatus {
    pub config_map: Option<String>,
    pub cron_job: Option<String>,
    pub last_schedule_time: Option<Time>,
}

#[derive(CustomResource, Serialize, Deserialize, Debug, PartialEq, Clone, JsonSchema, Builder)]
#[kube(
    group = "restic.anshulg.com",
    version = "v1alpha1",
    kind = "Backup",
    plural = "backups",
    derive = "PartialEq",
    status = "BackupStatus",
    shortname = "rb",
    category = "restic",
    printcolumn = r#"{"name": "Age", "type": "date", "jsonPath": ".metadata.creationTimestamp"}"#,
    printcolumn = r#"{"name": "Phase", "type": "string", "jsonPath": ".status.phase"}"#,
    namespaced
)]
#[serde(rename_all = "camelCase")]
pub struct BackupSpec {
    /// Restic Configuration
    pub restic: ResticConfig,

    /// Resticprofile Configuration
    pub restic_profile: Option<ResticProfileConfig>,

    /// Volume Backup
    pub volume: Option<VolumeBackup>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema, Builder)]
#[serde(rename_all = "camelCase")]
pub struct BackupStatus {
    #[serde(default)]
    pub phase: BackupPhase,
    pub config_map: Option<String>,
    pub job: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema, Default)]
pub enum BackupPhase {
    #[default]
    Pending,
    Running,
    Completed,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema, Builder)]
#[serde(rename_all = "camelCase")]
pub struct ResticConfig {
    /// The Restic Repository Configuration
    pub repository: Repository,
    /// Compression mode (only available for repository format version 2), one of (auto/off/max)
    #[serde(default)]
    #[builder(default)]
    pub compression: Compression,
    /// Set target pack size in MiB, created pack files may be larger
    pub pack_size: Option<u64>,
    /// Retention policy
    pub retention: Option<Retention>,
    /// Backup Options
    pub backup: Option<BackupOptions>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema, Builder)]
#[serde(rename_all = "camelCase")]
pub struct Repository {
    /// Repository Type
    pub r#type: RepositoryType,
    /// Repository URI. Do not include the repository type prefix (ex rest:...)
    pub uri: String,
    /// Secret to read the repository password from
    pub password: SecretKeySelector,
    /// Rest repository credentials
    pub rest_credentials: Option<RestCredentials>,
}

impl Repository {
    pub fn full_uri(&self) -> String {
        match self.r#type {
            RepositoryType::Rest => format!("rest:{}", self.uri),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone, Eq, PartialEq, Default, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub enum RepositoryType {
    #[default]
    Rest,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema, Builder)]
pub struct RestCredentials {
    pub username: SecretKeySelector,
    pub password: SecretKeySelector,
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone, Eq, PartialEq, Default, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum Compression {
    Off,
    #[default]
    Auto,
    Max,
}

impl Compression {
    pub fn as_str(&self) -> &str {
        match self {
            Compression::Off => "off",
            Compression::Auto => "auto",
            Compression::Max => "max",
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq, JsonSchema, Builder)]
#[serde(rename_all = "camelCase")]
pub struct Retention {
    #[serde(default)]
    pub after_backup: bool,
    #[serde(default)]
    pub before_backup: bool,

    pub keep_last: Option<u32>,
    pub keep_hourly: Option<u32>,
    pub keep_daily: Option<u32>,
    pub keep_weekly: Option<u32>,
    pub keep_monthly: Option<u32>,
    pub keep_yearly: Option<u32>,

    #[serde(default)]
    pub prune: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq, JsonSchema, Builder)]
#[serde(rename_all = "camelCase")]
pub struct BackupOptions {
    pub exclude: Option<Vec<String>>,
    #[serde(default)]
    pub exclude_caches: bool,
    pub exclude_if_present: Option<Vec<String>>,
    pub exclude_larger_than: Option<String>,
    pub iexclude: Option<Vec<String>>,
    // #[serde(default)]
    // pub ignore_ctime: bool,
    // #[serde(default)]
    // pub ignore_inode: bool,
    // #[serde(default)]
    // pub skip_if_unchanged: bool,
    pub tag: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema, Builder, Default)]
#[serde(rename_all = "camelCase")]
pub struct ResticProfileConfig {
    /// Docker Image to use for resticprofile
    pub image: Option<String>,
    /// Docker Image tag to use for resticprofile if `image` is not provided.
    /// Will use `latest` if not provided.
    pub version: Option<String>,
    /// Docker Image pull policy
    pub image_pull_policy: Option<String>,
    /// Arguments to the entrypoint. The container image's CMD is used if this is not provided. Variable references $(VAR_NAME) are expanded using the container's environment. If a variable cannot be resolved, the reference in the input string will be unchanged. Double $$ are reduced to a single $, which allows for escaping the $(VAR_NAME) syntax: i.e. "$$(VAR_NAME)" will produce the string literal "$(VAR_NAME)". Escaped references will never be expanded, regardless of whether the variable exists or not. Cannot be updated. More info: https://kubernetes.io/docs/tasks/inject-data-application/define-command-argument-container/#running-a-command-in-a-shell
    pub args: Option<Vec<String>>,
    /// Entrypoint array. Not executed within a shell. The container image's ENTRYPOINT is used if this is not provided. Variable references $(VAR_NAME) are expanded using the container's environment. If a variable cannot be resolved, the reference in the input string will be unchanged. Double $$ are reduced to a single $, which allows for escaping the $(VAR_NAME) syntax: i.e. "$$(VAR_NAME)" will produce the string literal "$(VAR_NAME)". Escaped references will never be expanded, regardless of whether the variable exists or not. Cannot be updated. More info: https://kubernetes.io/docs/tasks/inject-data-application/define-command-argument-container/#running-a-command-in-a-shell
    pub command: Option<Vec<String>>,
    /// List of environment variables to set in the container.
    pub env: Option<Vec<EnvVar>>,
    /// List of sources to populate environment variables in the container. The keys defined within a source must be a C_IDENTIFIER. All invalid keys will be reported as an event when the container is starting. When a key exists in multiple sources, the value associated with the last source will take precedence. Values defined by an Env with a duplicate key will take precedence.
    pub env_from: Option<Vec<EnvFromSource>>,
    /// Compute Resources required by this container. More info: https://kubernetes.io/docs/concepts/configuration/manage-resources-containers/
    pub resources: Option<ResourceRequirements>,
    /// RestartPolicy defines the restart behavior of individual containers in a pod.
    pub restart_policy: Option<String>,
    /// SecurityContext defines the security options the container should be run with. If set, the fields of SecurityContext override the equivalent fields of PodSecurityContext. More info: https://kubernetes.io/docs/tasks/configure-pod-container/security-context/
    pub security_context: Option<SecurityContext>,

    /// If specified, the pod's scheduling constraints
    pub affinity: Option<Affinity>,
    /// NodeSelector is a selector which must be true for the pod to fit on a node. Selector which must match a node's labels for the pod to be scheduled on that node. More info: https://kubernetes.io/docs/concepts/configuration/assign-pod-node/
    pub node_selector: Option<std::collections::BTreeMap<String, String>>,
    /// ServiceAccountName is the name of the ServiceAccount to use to run this pod. More info: https://kubernetes.io/docs/tasks/configure-pod-container/configure-service-account/
    pub service_account_name: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema, Builder)]
#[serde(rename_all = "camelCase")]
pub struct VolumeBackup {
    pub mounts: Vec<VolumeMount>,
    pub volumes: Vec<Volume>,
}

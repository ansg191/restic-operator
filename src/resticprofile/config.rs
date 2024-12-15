use std::collections::HashMap;

use bon::Builder;
use serde::{Deserialize, Serialize};

pub const DEFAULT_PROFILE: &str = "default";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Builder, Default)]
#[non_exhaustive]
pub struct ResticProfileConfig {
    #[builder(default)]
    pub version: ResticProfileVersion,
    pub global: Option<ResticProfileGlobal>,
    #[serde(flatten)]
    #[builder(default)]
    pub profiles: HashMap<String, ResticProfileProfile>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[non_exhaustive]
pub enum ResticProfileVersion {
    #[default]
    #[serde(rename = "1")]
    V1,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Builder, Default)]
#[non_exhaustive]
#[serde(rename_all = "kebab-case")]
pub struct ResticProfileGlobal {
    /// Minimum available memory (in MB) required to run any commands.
    pub min_memory: Option<u64>,
    /// Time to wait before trying to get a lock on a restic repository.
    pub restic_lock_retry_after: Option<String>,
    /// The age an unused lock on a restic repository must have at least before resticprofile attempts to unlock.
    pub restic_stale_lock_age: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Builder, Default)]
#[non_exhaustive]
#[serde(rename_all = "kebab-case")]
pub struct ResticProfileProfile {
    /// File to load root certificates from (default: use system certificates or $RESTIC_CACERT).
    pub cacert: Option<String>,
    /// Set the cache directory. (default: use system default cache directory).
    pub cache_dir: Option<String>,
    /// Compression mode (only available for repository format version 2), one of (auto/off/max) (default: $RESTIC_COMPRESSION).
    pub compression: Option<String>,
    /// Set a http user agent for outgoing http requests.
    pub http_user_agent: Option<String>,
    /// Skip TLS certificate verification when connecting to the repository (insecure).
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    #[builder(default)]
    pub insecure_tls: bool,
    /// Limits downloads to a maximum rate in KiB/s. (default: unlimited).
    pub limit_download: Option<u64>,
    /// Limits uploads to a maximum rate in KiB/s. (default: unlimited).
    pub limit_upload: Option<u64>,
    /// Do not use a local cache
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    #[builder(default)]
    pub no_cache: bool,
    /// Skip additional verification of data before upload (see documentation)
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    #[builder(default)]
    pub no_extra_verify: bool,
    /// set target pack size in MiB, created pack files may be larger (default: $RESTIC_PACK_SIZE).
    pub pack_size: Option<u64>,
    /// File to read the repository password from.
    pub password_file: Option<String>,
    /// Repository to backup to or restore from (default: $RESTIC_REPOSITORY).
    pub repository: Option<String>,
    /// Path to a file containing PEM encoded TLS client certificate and private key (default: $RESTIC_TLS_CLIENT_CERT).
    pub tls_client_cert: Option<String>,
    /// Be verbose
    pub verbose: Option<u8>,

    /// This section configures restic command `backup`.
    pub backup: Option<ResticProfileProfileBackup>,
    /// This section configures restic command `forget`.
    pub retention: Option<ResticProfileProfileRetention>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Builder, Default)]
#[non_exhaustive]
#[serde(rename_all = "kebab-case")]
pub struct ResticProfileProfileBackup {
    /// Check the repository after the backup command succeeded.
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    #[builder(default)]
    pub check_after: bool,
    /// Check the repository before starting the backup command.
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    #[builder(default)]
    pub check_before: bool,
    /// Do not fail the backup when some files could not be read.
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    #[builder(default)]
    pub no_error_on_warning: bool,
    /// The paths to backup. Examples: /opt/, /home/user/, C:\Users\User\Documents.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[builder(default)]
    pub source: Vec<String>,
    /// Exclude a pattern.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[builder(default)]
    pub exclude: Vec<String>,
    /// Excludes cache directories that are marked with a CACHEDIR.TAG file.
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    #[builder(default)]
    pub exclude_caches: bool,
    /// Takes filename[:header], exclude contents of directories containing filename (except filename itself) if header of that file is as provided.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[builder(default)]
    pub exclude_if_present: Vec<String>,
    /// Max size of the files to be backed up (allowed suffixes: k/K, m/M, g/G, t/T).
    pub exclude_larger_than: Option<String>,
    /// Set the hostname for the snapshot manually.
    pub host: Option<String>,
    /// Same as –exclude pattern but ignores the casing of filenames.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[builder(default)]
    pub iexclude: Vec<String>,
    /// Add tags for the new snapshot in the format tag[,tag,…]. Boolean true is unsupported in section “backup”. Examples: false, "tag".
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[builder(default)]
    pub tag: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Builder, Default)]
#[non_exhaustive]
#[serde(rename_all = "kebab-case")]
pub struct ResticProfileProfileRetention {
    /// Apply retention after the backup command succeeded.
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    #[builder(default)]
    pub after_backup: bool,
    /// Apply retention before starting the backup command
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    #[builder(default)]
    pub before_backup: bool,
    #[builder(default)]
    pub host: bool,

    pub keep_last: Option<u32>,
    pub keep_hourly: Option<u32>,
    pub keep_daily: Option<u32>,
    pub keep_weekly: Option<u32>,
    pub keep_monthly: Option<u32>,
    pub keep_yearly: Option<u32>,

    /// Automatically run the ‘prune’ command if snapshots have been removed.
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    #[builder(default)]
    pub prune: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default() {
        let config = ResticProfileConfig::default();

        let output = toml::to_string(&config).unwrap();
        assert_eq!(
            output,
            r#"version = "1"
"#
        );
    }

    #[test]
    fn test_global() {
        let config = ResticProfileConfig::builder()
            .global(
                ResticProfileGlobal::builder()
                    .min_memory(100)
                    .restic_lock_retry_after("1m".to_owned())
                    .restic_stale_lock_age("1h".to_owned())
                    .build(),
            )
            .build();

        let output = toml::to_string(&config).unwrap();
        assert_eq!(
            output,
            r#"version = "1"

[global]
min-memory = 100
restic-lock-retry-after = "1m"
restic-stale-lock-age = "1h"
"#
        );
    }

    #[test]
    fn test_profile() {
        let config = ResticProfileConfig::builder()
            .profiles(HashMap::from([(
                "default".to_owned(),
                ResticProfileProfile::builder()
                    .cacert("/etc/ssl/ca.crt".to_owned())
                    .cache_dir("/var/cache/restic".to_owned())
                    .compression("auto".to_owned())
                    .http_user_agent("resticprofile/0.1".to_owned())
                    .insecure_tls(false)
                    .backup(
                        ResticProfileProfileBackup::builder()
                            .source(vec!["/opt".to_owned()])
                            .build(),
                    )
                    .retention(
                        ResticProfileProfileRetention::builder()
                            .after_backup(true)
                            .keep_last(10)
                            .build(),
                    )
                    .build(),
            )]))
            .build();

        let output = toml::to_string(&config).unwrap();
        assert_eq!(
            output,
            r#"version = "1"

[default]
cacert = "/etc/ssl/ca.crt"
cache-dir = "/var/cache/restic"
compression = "auto"
http-user-agent = "resticprofile/0.1"

[default.backup]
source = ["/opt"]

[default.retention]
after-backup = true
host = false
keep-last = 10
"#
        );
    }
}

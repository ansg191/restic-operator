use std::collections::{BTreeMap, HashMap};

use config::{
    ResticProfileConfig, ResticProfileProfile, ResticProfileProfileBackup,
    ResticProfileProfileRetention, DEFAULT_PROFILE,
};
use k8s_openapi::{api::core::v1::ConfigMap, apimachinery::pkg::apis::meta::v1::OwnerReference};
use kube::{api::ObjectMeta, Api, Client, ResourceExt};
use restic_crd::BackupSpec;

use crate::{
    deploy::{Deployable, Labels},
    Error,
};

pub mod config;

const PASSWORD_FILE_PATH: &str = "/resticprofile/password.txt";

#[derive(Debug, Clone)]
pub struct ResticProfile {
    name: String,
    ns: String,
    config: ResticProfileConfig,
}

impl ResticProfile {
    pub fn new(ns: String, name: impl Into<String>, backup: &BackupSpec) -> Self {
        let backup_name = name.into();
        let name = format!("{backup_name}-profile");
        let config = create_config(backup_name, backup);
        ResticProfile { name, ns, config }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    async fn get(&self, client: Client) -> Result<Option<ConfigMap>, Error> {
        let api: Api<ConfigMap> = Api::namespaced(client, &self.ns);
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

impl Deployable for ResticProfile {
    type Error = Error;

    async fn create<O>(&self, client: Client, owner: &O, labels: Labels) -> Result<(), Self::Error>
    where
        O: kube::Resource<DynamicType = ()> + Send + Sync,
    {
        let config = toml::to_string(&self.config)?;
        let data = BTreeMap::from([("profiles.toml".to_owned(), config)]);

        let config_map = ConfigMap {
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
                ..ObjectMeta::default()
            },
            data: Some(data),
            ..ConfigMap::default()
        };

        let api: Api<ConfigMap> = Api::namespaced(client, &self.ns);
        api.create(&Default::default(), &config_map).await?;

        Ok(())
    }

    async fn delete(&self, client: Client) -> Result<(), Self::Error> {
        let config_map = self.get(client.clone()).await?;
        if let Some(config_map) = config_map {
            let api: Api<ConfigMap> = Api::namespaced(client, &self.ns);
            api.delete(&config_map.name_any(), &Default::default())
                .await?;
        }
        Ok(())
    }
}

fn create_config(name: String, backup: &BackupSpec) -> ResticProfileConfig {
    let paths = extract_paths(backup);

    let backup_conf = backup.restic.backup.as_ref().map(|b| {
        ResticProfileProfileBackup::builder()
            .source(paths)
            .maybe_exclude(b.exclude.clone())
            .exclude_caches(b.exclude_caches)
            .maybe_exclude_if_present(b.exclude_if_present.clone())
            .maybe_exclude_larger_than(b.exclude_larger_than.clone())
            .maybe_iexclude(b.iexclude.clone())
            .maybe_tag(b.tag.clone())
            .host(name)
            .build()
    });

    let retention = backup.restic.retention.as_ref().map(|r| {
        ResticProfileProfileRetention::builder()
            .after_backup(r.after_backup)
            .before_backup(r.before_backup)
            .maybe_keep_last(r.keep_last)
            .maybe_keep_hourly(r.keep_hourly)
            .maybe_keep_daily(r.keep_daily)
            .maybe_keep_weekly(r.keep_weekly)
            .maybe_keep_monthly(r.keep_monthly)
            .maybe_keep_yearly(r.keep_yearly)
            .prune(r.prune)
            .build()
    });

    let profile = ResticProfileProfile::builder()
        .compression(backup.restic.compression.as_str().to_owned())
        .repository(backup.restic.repository.full_uri())
        .password_file(PASSWORD_FILE_PATH.to_owned())
        .maybe_backup(backup_conf)
        .maybe_retention(retention)
        .build();

    ResticProfileConfig::builder()
        .profiles(HashMap::from([(DEFAULT_PROFILE.to_owned(), profile)]))
        .build()
}

fn extract_paths(backup: &BackupSpec) -> Vec<String> {
    if let Some(vol_backup) = &backup.volume {
        vol_backup
            .mounts
            .iter()
            .map(|m| m.mount_path.clone())
            .collect()
    } else {
        Vec::new()
    }
}

#[cfg(test)]
mod tests {
    use k8s_openapi::{
        api::core::v1::VolumeMount, apimachinery::pkg::apis::meta::v1::OwnerReference,
    };
    use restic_crd::{Backup, BackupSpec, Repository, ResticConfig, VolumeBackup};

    use super::*;

    #[test]
    fn test_extract_paths() {
        let spec = BackupSpec::builder()
            .restic(
                ResticConfig::builder()
                    .repository(
                        Repository::builder()
                            .r#type(restic_crd::RepositoryType::Rest)
                            .uri("https://example.com".to_owned())
                            .password(Default::default())
                            .build(),
                    )
                    .build(),
            )
            .volume(
                VolumeBackup::builder()
                    .mounts(vec![VolumeMount {
                        mount_path: "/mnt/data".to_owned(),
                        ..Default::default()
                    }])
                    .volumes(Vec::new())
                    .build(),
            )
            .build();
        let backup = Backup::new("test", spec);

        let paths = extract_paths(&backup.spec);
        assert_eq!(paths, vec!["/mnt/data".to_owned()]);
    }

    #[tokio::test]
    // #[cfg_attr(
    //     not(feature = "integration-tests"),
    //     ignore = "uses k8s current-context"
    // )]
    #[ignore = "TODO FIX: incorrect naming"]
    async fn test_resticprofile() {
        const NAME: &str = "test-resticprofile";
        const NS: &str = "default";
        let client = Client::try_default().await.unwrap();

        let spec = BackupSpec::builder()
            .restic(
                ResticConfig::builder()
                    .repository(
                        Repository::builder()
                            .r#type(restic_crd::RepositoryType::Rest)
                            .uri("https://example.com".to_owned())
                            .password(Default::default())
                            .build(),
                    )
                    .build(),
            )
            .build();
        let _backup = Backup::new(NAME, spec);

        // let _profile = ResticProfile::new(NS.to_owned(), &backup);
        // profile
        // .create(client.clone(), Labels::new("test"))
        // .await
        // .unwrap();

        let api: Api<ConfigMap> = Api::namespaced(client, NS);
        let cm = api.get(NAME).await.unwrap();
        assert_eq!(cm.name_any(), NAME);
        assert_eq!(
            cm.owner_references(),
            vec![OwnerReference {
                name: "test".to_owned(),
                ..Default::default()
            }]
        );
    }
}

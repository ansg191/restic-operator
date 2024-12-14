use k8s_openapi::api::{
    batch::v1::JobSpec,
    core::v1::{
        Affinity, ConfigMapVolumeSource, Container, EnvFromSource, EnvVar, EnvVarSource, PodSpec,
        PodTemplateSpec, ResourceRequirements, SecretVolumeSource, SecurityContext, Volume,
        VolumeMount,
    },
};
use restic_crd::{BackupSpec, ResticProfileConfig};

const DEFAULT_RESTIC_IMAGE: &str = "creativeprojects/resticprofile";

#[derive(Debug, Clone)]
pub struct BackupJobSpec {
    image: String,
    image_pull_policy: Option<String>,
    args: Option<Vec<String>>,
    command: Option<Vec<String>>,
    env: Vec<EnvVar>,
    env_from: Vec<EnvFromSource>,
    resources: Option<ResourceRequirements>,
    security_context: Option<SecurityContext>,
    affinity: Option<Affinity>,
    node_selector: Option<std::collections::BTreeMap<String, String>>,
    service_account_name: Option<String>,
    volume_mounts: Vec<VolumeMount>,
    volumes: Vec<Volume>,
}

impl BackupJobSpec {
    pub fn new(backup: &BackupSpec, config_name: impl Into<String>) -> Self {
        let mut rpcfg = backup.restic_profile.clone().unwrap_or_default();
        let image = get_image(&mut rpcfg);
        let env = fill_env(backup, &mut rpcfg);
        let (volume_mounts, volumes) = fill_volume_mounts(backup, config_name);

        Self {
            image,
            image_pull_policy: rpcfg.image_pull_policy.take(),
            args: rpcfg.args.take(),
            command: rpcfg.command.take(),
            env,
            env_from: rpcfg.env_from.take().unwrap_or_default(),
            resources: rpcfg.resources.take(),
            security_context: rpcfg.security_context.take(),
            affinity: rpcfg.affinity.take(),
            node_selector: rpcfg.node_selector.take(),
            service_account_name: rpcfg.service_account_name.take(),
            volume_mounts,
            volumes,
        }
    }
}

impl From<BackupJobSpec> for JobSpec {
    fn from(value: BackupJobSpec) -> Self {
        Self {
            suspend: Some(false),
            template: PodTemplateSpec {
                spec: Some(PodSpec {
                    affinity: value.affinity,
                    containers: vec![Container {
                        name: "restic-backup".to_owned(),
                        args: value.args,
                        command: value.command,
                        env: Some(value.env),
                        env_from: Some(value.env_from),
                        image: Some(value.image),
                        image_pull_policy: value.image_pull_policy,
                        resources: value.resources,
                        security_context: value.security_context,
                        volume_mounts: Some(value.volume_mounts),
                        ..Default::default()
                    }],
                    restart_policy: Some("OnFailure".to_string()),
                    node_selector: value.node_selector,
                    service_account_name: value.service_account_name,
                    volumes: Some(value.volumes),
                    ..Default::default()
                }),
                ..Default::default()
            },
            ..Default::default()
        }
    }
}

fn get_image(cfg: &mut ResticProfileConfig) -> String {
    cfg.image.take().unwrap_or_else(|| {
        format!(
            "{}:{}",
            DEFAULT_RESTIC_IMAGE,
            cfg.version.as_deref().unwrap_or("latest")
        )
    })
}

fn fill_env(backup: &BackupSpec, rpcfg: &mut ResticProfileConfig) -> Vec<EnvVar> {
    let mut env = rpcfg.env.take().unwrap_or_default();

    if let Some(rest_creds) = &backup.restic.repository.rest_credentials {
        env.push(EnvVar {
            name: "RESTIC_REST_USERNAME".to_string(),
            value_from: Some(EnvVarSource {
                secret_key_ref: Some(rest_creds.username.clone()),
                ..Default::default()
            }),
            ..Default::default()
        });
        env.push(EnvVar {
            name: "RESTIC_REST_PASSWORD".to_string(),
            value_from: Some(EnvVarSource {
                secret_key_ref: Some(rest_creds.password.clone()),
                ..Default::default()
            }),
            ..Default::default()
        });
    }

    env
}

fn fill_volume_mounts(
    backup: &BackupSpec,
    config_name: impl Into<String>,
) -> (Vec<VolumeMount>, Vec<Volume>) {
    let mut mounts = Vec::new();
    let mut volumes = Vec::new();

    // Add volume mount for resticprofile config
    mounts.push(VolumeMount {
        mount_path: "/resticprofile/profiles.toml".to_owned(),
        name: "profile".to_owned(),
        sub_path: Some("profiles.toml".to_owned()),
        ..Default::default()
    });
    volumes.push(Volume {
        name: "profile".to_owned(),
        config_map: Some(ConfigMapVolumeSource {
            name: config_name.into(),
            ..Default::default()
        }),
        ..Default::default()
    });

    // Add volume mount for restic repository password
    mounts.push(VolumeMount {
        mount_path: "/resticprofile/password.txt".to_owned(),
        name: "restic-password".to_owned(),
        sub_path: Some(backup.restic.repository.password.key.clone()),
        ..Default::default()
    });
    volumes.push(Volume {
        name: "restic-password".to_owned(),
        secret: Some(SecretVolumeSource {
            secret_name: Some(backup.restic.repository.password.name.clone()),
            ..Default::default()
        }),
        ..Default::default()
    });

    // Add other volume mounts
    if let Some(vol_backup) = &backup.volume {
        mounts.extend_from_slice(&vol_backup.mounts);
        volumes.extend_from_slice(&vol_backup.volumes);
    };

    (mounts, volumes)
}

#[cfg(test)]
mod tests {
    use k8s_openapi::api::core::v1::SecretKeySelector;
    use restic_crd::{Repository, RepositoryType, RestCredentials, ResticConfig, VolumeBackup};

    use super::*;

    const CONFIG_NAME: &str = "test-config";

    fn create_backup() -> BackupSpec {
        BackupSpec {
            restic: ResticConfig::builder()
                .repository(Repository {
                    r#type: RepositoryType::Rest,
                    uri: "https://example.com".to_string(),
                    rest_credentials: Some(RestCredentials {
                        username: SecretKeySelector {
                            name: "restic-secret".to_string(),
                            key: "username".to_string(),
                            ..Default::default()
                        },
                        password: SecretKeySelector {
                            name: "restic-secret".to_string(),
                            key: "password".to_string(),
                            ..Default::default()
                        },
                    }),
                    password: SecretKeySelector {
                        name: "restic-password".to_string(),
                        key: "password.txt".to_string(),
                        ..Default::default()
                    },
                })
                .build(),
            volume: None,
            restic_profile: Some(ResticProfileConfig {
                image: Some("custom/restic:latest".to_string()),
                version: Some("v1.0.0".to_string()),
                ..Default::default()
            }),
        }
    }

    #[test]
    fn test_jobspec_new() {
        let backup = create_backup();
        let job_spec = BackupJobSpec::new(&backup, CONFIG_NAME);

        assert_eq!(job_spec.image, "custom/restic:latest");
        assert_eq!(job_spec.env.len(), 2);
        assert_eq!(job_spec.volume_mounts.len(), 2);
        assert_eq!(job_spec.volumes.len(), 2);
    }

    #[test]
    fn test_default_image() {
        let mut backup = create_backup();
        backup.restic_profile.as_mut().unwrap().image = None;
        let job = BackupJobSpec::new(&backup, CONFIG_NAME);

        assert_eq!(job.image, "creativeprojects/resticprofile:v1.0.0");
    }

    #[test]
    fn test_default_image_tag() {
        let mut backup = create_backup();
        backup.restic_profile.as_mut().unwrap().image = None;
        backup.restic_profile.as_mut().unwrap().version = None;
        let job = BackupJobSpec::new(&backup, CONFIG_NAME);

        assert_eq!(job.image, "creativeprojects/resticprofile:latest");
    }

    #[test]
    fn test_fill_env_with_no_credentials() {
        let mut backup = create_backup();
        backup.restic.repository.rest_credentials = None;
        let mut rpcfg = backup.restic_profile.clone().unwrap_or_default();
        let env = fill_env(&backup, &mut rpcfg);

        assert!(env.is_empty());
    }

    #[test]
    fn test_fill_volume_mounts_with_no_volume() {
        let backup = create_backup();
        let config_name = "test-config";
        let (volume_mounts, volumes) = fill_volume_mounts(&backup, config_name);

        assert_eq!(volume_mounts.len(), 2);
        assert_eq!(volumes.len(), 2);
    }

    #[test]
    fn test_fill_volume_mounts_with_volume() {
        let mut backup = create_backup();
        backup.volume = Some(VolumeBackup {
            mounts: vec![VolumeMount {
                mount_path: "/data".to_string(),
                name: "data-volume".to_string(),
                ..Default::default()
            }],
            volumes: vec![Volume {
                name: "data-volume".to_string(),
                ..Default::default()
            }],
        });
        let config_name = "test-config";
        let (volume_mounts, volumes) = fill_volume_mounts(&backup, config_name);

        assert_eq!(volume_mounts.len(), 3);
        assert_eq!(volumes.len(), 3);
    }
}

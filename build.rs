use std::io::Write;

use kube::CustomResourceExt;
use restic_crd::{Backup, ScheduledBackup};

fn main() {
    // Make sure target directory exists
    std::fs::create_dir_all("crds").unwrap();

    // Generate CRD YAML
    let backup_crd = serde_yaml::to_string(&Backup::crd()).unwrap();
    let scheduled_backup_crd = serde_yaml::to_string(&ScheduledBackup::crd()).unwrap();

    // Output CRD to `restic-operator-VERSION.yaml` in target directory
    let version = env!("CARGO_PKG_VERSION");
    let filename = format!("crds/restic-operator-{}.yaml", version);
    let mut file = std::fs::File::create(&filename).unwrap();
    file.set_len(0).unwrap();
    file.write_all(backup_crd.as_bytes()).unwrap();
    file.write_all(b"---\n").unwrap();
    file.write_all(scheduled_backup_crd.as_bytes()).unwrap();
}

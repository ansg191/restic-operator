use std::io::Write;

use cargo_metadata::MetadataCommand;
use clap::{Parser, Subcommand};
use kube::CustomResourceExt;
use restic_crd::{Backup, ScheduledBackup};

const PACKAGE_NAME: &str = "restic-operator";

#[derive(Debug, Parser)]
struct Args {
    #[clap(subcommand)]
    cmd: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Generate CRD YAML
    GenerateCrd,
}

fn main() {
    let args = Args::parse();

    match args.cmd {
        Command::GenerateCrd => generate_crd(),
    }
}

fn generate_crd() {
    // Make sure target directory exists
    std::fs::create_dir_all("crds").unwrap();

    // Generate CRD YAML
    let backup_crd = serde_yaml::to_string(&Backup::crd()).unwrap();
    let scheduled_backup_crd = serde_yaml::to_string(&ScheduledBackup::crd()).unwrap();

    // Get operator version from Cargo.toml
    let metadata = MetadataCommand::new()
        .exec()
        .expect("Should be able to get metadata");
    let package = metadata
        .packages
        .iter()
        .find(|pkg| pkg.name == PACKAGE_NAME)
        .expect("restic-operator package should be in workspace");
    let version = package.version.to_string();
    eprintln!("restic-operator version: {version}");

    // Output CRD to `restic-operator-VERSION.yaml` in target directory
    let filename = format!("crds/restic-operator-{version}.yaml");
    eprintln!("Writing CRD to {filename}");

    let mut file = std::fs::File::create(&filename).unwrap();
    file.set_len(0).unwrap();
    file.write_all(backup_crd.as_bytes()).unwrap();
    file.write_all(b"---\n").unwrap();
    file.write_all(scheduled_backup_crd.as_bytes()).unwrap();
    file.flush().unwrap();
    eprintln!("Done");
}

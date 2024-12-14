use std::{sync::Arc, time::Duration};

use futures::StreamExt;
use kube::{
    runtime::{controller::Action, watcher::Config, Controller},
    Api, Client, Resource, ResourceExt,
};
use restic_crd::ScheduledBackup;
use tracing::{error, info};

use crate::{
    context::ContextData,
    deploy::{Deployable, Labels},
    finalizer::{self, FINALIZER},
    Error,
};

mod cronjob;
mod deploy;

pub async fn run_controller(client: Client) {
    let crd_api: Api<ScheduledBackup> = Api::all(client.clone());
    let context: Arc<ContextData> = Arc::new(ContextData::new(client));

    Controller::new(crd_api, Config::default())
        .run(reconcile, on_error, context)
        .for_each(|reconciliation_result| async move {
            match reconciliation_result {
                Ok(echo_resource) => {
                    info!("Reconciliation successful. Resource: {:?}", echo_resource);
                }
                Err(reconciliation_err) => {
                    error!(%reconciliation_err, "Reconciliation error")
                }
            }
        })
        .await;
}

async fn reconcile(
    backup: Arc<ScheduledBackup>,
    context: Arc<ContextData>,
) -> Result<Action, Error> {
    let client = context.client.clone();

    let ns = backup.namespace().ok_or(Error::MissingNamespace)?;
    let name = backup.name_any();

    match determine_action(&backup) {
        ScheduledBackupAction::Create => {
            // Add the finalizer to the resource
            finalizer::add(
                &Api::<ScheduledBackup>::namespaced(client.clone(), &ns),
                &name,
            )
            .await?;

            // Create the deployment
            let deployment = deploy::ScheduledBackupDeployment::new(ns, &backup);
            let labels = Labels::new(name);
            deployment.create(client, &*backup, labels).await?;

            Ok(Action::requeue(Duration::from_secs(10)))
        }
        ScheduledBackupAction::Delete => {
            // Delete the deployment
            let deployment = deploy::ScheduledBackupDeployment::new(ns.clone(), &backup);
            deployment.delete(client.clone()).await?;

            // Remove the finalizer from the resource
            finalizer::remove(
                &Api::<ScheduledBackup>::namespaced(client.clone(), &ns),
                &name,
            )
            .await?;
            Ok(Action::requeue(Duration::from_secs(10)))
        }
        ScheduledBackupAction::Noop => Ok(Action::requeue(Duration::from_secs(10))),
    }
}

fn determine_action(backup: &ScheduledBackup) -> ScheduledBackupAction {
    if backup.meta().deletion_timestamp.is_some() {
        ScheduledBackupAction::Delete
    } else if backup
        .meta()
        .finalizers
        .as_ref()
        .map_or(true, |f| !f.iter().any(|x| x == FINALIZER))
    {
        ScheduledBackupAction::Create
    } else {
        ScheduledBackupAction::Noop
    }
}

fn on_error(backup: Arc<ScheduledBackup>, error: &Error, _context: Arc<ContextData>) -> Action {
    error!("Reconciliation error:\n{:?}.\n{:?}", error, backup);
    Action::requeue(Duration::from_secs(5))
}

/// Possible actions to take on a [`ScheduledBackup`] resource
enum ScheduledBackupAction {
    /// Create the sub-resources for the backup
    Create,
    /// Delete the sub-resources for the backup
    Delete,
    /// No operation required. Update the status if needed.
    Noop,
}

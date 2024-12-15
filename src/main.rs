use kube::Client;
use tracing::{info, level_filters::LevelFilter};
use tracing_subscriber::EnvFilter;

mod backup;
mod context;
mod deploy;
mod error;
mod finalizer;
mod jobspec;
mod resticprofile;
mod schedule;

pub use error::Error;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        .init();
    std::panic::set_hook(Box::new(tracing_panic::panic_hook));

    let k8s_client = Client::try_default()
        .await
        .expect("Expected a valid KUBECONFIG environment variable.");

    info!("Starting up...");

    let signal = tokio::signal::ctrl_c();
    let backup_fut = tokio::spawn(backup::run_controller(k8s_client.clone()));
    let schedule_fut = tokio::spawn(schedule::run_controller(k8s_client.clone()));

    info!("Controllers started.");

    tokio::select! {
        _ = signal => {}
        _ = backup_fut => {}
        _ = schedule_fut => {}
    }

    info!("Successfully shut down.")
}

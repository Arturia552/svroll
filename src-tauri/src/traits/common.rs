use std::fmt::Debug;

use anyhow::{Error, Result};
use tokio::task::JoinHandle;

use crate::{benchmark_param::BenchmarkConfig, model::tauri_com::Task};

pub trait Client<T, C>: Send + Sync
where
    T: Debug,
{
    type Item;

    fn setup_clients(
        &self,
        config: &BenchmarkConfig<T, C>,
    ) -> impl std::future::Future<Output = Result<Vec<Self::Item>, Error>> + Send;

    fn wait_for_connections(
        &self,
        clients: &mut [Self::Item],
    ) -> impl std::future::Future<Output = ()> + Send;

    fn on_connect_success(
        &self,
        client: &mut Self::Item,
    ) -> impl std::future::Future<Output = Result<(), Error>> + Send;
    fn spawn_message(
        &self,
        clients: Vec<Self::Item>,
        task: &Task,
        config: &BenchmarkConfig<T, C>,
    ) -> impl std::future::Future<Output = Result<Vec<JoinHandle<()>>, Error>> + Send;
}

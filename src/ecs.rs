use crate::config::get_default_config;
use crate::errors::AwsError;
pub use aws_sdk_ecs::model::KeyValuePair;
use aws_sdk_ecs::model::{
    AssignPublicIp, AwsVpcConfiguration, ContainerOverride, LaunchType, NetworkConfiguration,
    PropagateTags, Task, TaskOverride,
};
use aws_sdk_ecs::output::DescribeTasksOutput;
use aws_sdk_ecs::RetryConfig;
use aws_types::SdkConfig;
use serde::Deserialize;

/// An Elastic Container Service client.
pub struct EcsClient {
    /// The underlying ECS client from the AWS SDK.
    pub client: aws_sdk_ecs::Client,
}

/// Creates an ECS client using the default AWS SDK config.
pub async fn create_default_ecs_client() -> EcsClient {
    let shared_config = get_default_config().await;
    EcsClient {
        client: aws_sdk_ecs::Client::new(&shared_config),
    }
}

impl EcsClient {
    /// Returns an ECS client built from `config`.
    ///
    /// In the retry config for requests, the maximum number of attempts is set to 2.
    pub fn new(config: &SdkConfig) -> EcsClient {
        let ecs_config = aws_sdk_ecs::config::Builder::from(config)
            .retry_config(RetryConfig::new().with_max_attempts(2))
            .build();
        let client = aws_sdk_ecs::Client::from_conf(ecs_config);
        EcsClient { client }
    }

    /// Launches a task definition.
    ///
    /// # Errors
    ///
    /// If the task cannot be started, for instance because of an invalid parameter, then an error is returned.
    ///
    /// # Note
    ///
    /// Some settings are currently hardcoded:
    ///
    /// * the launch type is `Fargate`
    /// * tags are propagated from the task definition
    ///
    pub async fn launch_task<T: EcsOverrideEnv>(
        &self,
        task_name: &str,
        container_name: &str,
        cluster_name: &str,
        network_config: EcsNetworkConfiguration,
        override_config: T,
    ) -> Result<(), AwsError> {
        let task_override = TaskOverride::builder()
            .container_overrides(
                ContainerOverride::builder()
                    .set_environment(Some(override_config.create_ecs_override_env()))
                    .name(container_name.to_string())
                    .build(),
            )
            .build();

        match self
            .client
            .run_task()
            .cluster(cluster_name.to_string())
            .count(1)
            .launch_type(LaunchType::Fargate)
            .network_configuration(network_config.into())
            .task_definition(task_name.to_string())
            .overrides(task_override)
            .propagate_tags(PropagateTags::TaskDefinition)
            .send()
            .await
        {
            Ok(_) => Ok(()),
            Err(e) => {
                tracing::error!(error=%e, "failed to start task {}", e);
                Err(AwsError::new(
                    format!("failed to start task {}", e).as_str(),
                ))
            }
        }
    }

    /// Retrieves a task override environment from the provided task ARN and cluster.
    ///
    /// # Errors
    ///
    /// If the tasks can't be retrieved or if the first task doesn't contain any task override, container override, or override environment, then an error is returned.
    pub async fn get_task_override_environment(
        &self,
        cluster: &str,
        task_arn: &str,
    ) -> Result<Vec<KeyValuePair>, AwsError> {
        let tasks_output = self
            .client
            .describe_tasks()
            .cluster(cluster)
            .tasks(task_arn)
            .send()
            .await;
        match tasks_output {
            Ok(DescribeTasksOutput {
                tasks: Some(tasks), ..
            }) => match get_override_environment_from_tasks(&tasks) {
                Some(description) => Ok(description),
                None => Err(AwsError::new(
                    "could not extract override environment from tasks",
                )),
            },
            Ok(DescribeTasksOutput { tasks: None, .. }) => Err(AwsError::new("no tasks found")),
            Err(e) => Err(AwsError::new(format!("failed to get tasks {}", e).as_str())),
        }
    }
}

fn get_override_environment_from_tasks(tasks: &[Task]) -> Option<Vec<KeyValuePair>> {
    tasks
        .first()?
        .overrides()?
        .container_overrides()?
        .first()?
        .environment()
        .map(|env| env.to_vec())
}

/// A trait to make data structures convertible into key-value pairs used to override containers environments.
pub trait EcsOverrideEnv {
    /// Creates the vector of key-value pairs.
    ///
    /// Example taken from the `kleanads-build-spawner` project:
    ///
    /// ```
    /// use aws_sdk_ecs::model::KeyValuePair;
    /// use aws_lib::ecs::EcsOverrideEnv;
    ///
    /// struct KleanadsBuilderConfig {
    ///     scope_config_id: String,
    ///     debug: bool,
    /// }
    ///
    /// impl EcsOverrideEnv for KleanadsBuilderConfig {
    ///     fn create_ecs_override_env(self) -> Vec<KeyValuePair> {
    ///         vec![
    ///             KeyValuePair::builder()
    ///                 .name("KLEANADS_SCOPE_BUILD_CONFIG_ID".to_string())
    ///                 .value(self.scope_config_id)
    ///                 .build(),
    ///             KeyValuePair::builder()
    ///                 .name("DEBUG".to_string())
    ///                 .value(self.debug.to_string())
    ///                 .build(),
    ///         ]
    ///     }
    /// }
    /// ```
    fn create_ecs_override_env(self) -> Vec<KeyValuePair>;
}

/// Simplified network configuration for ECS launch tasks
///
/// This configuration can be parameterized with subnets only. Internally the `AssignPublicIp` parameter is always enabled.
///
/// # Examples
///
/// ```
/// use aws_lib::ecs::EcsNetworkConfiguration;
///
/// let subnets = vec!["subnet-0941bb6933678e431".to_string(), "subnet-06581bd8767347cd0".to_string()];
/// let network_configuration = EcsNetworkConfiguration::new(subnets);
/// ```
///
/// Since this structure implements `Deserialize`, you can also create your configuration from environment variables, for instance using the `envy` crate:
///
/// ```
/// use std::env;
/// use aws_lib::ecs::EcsNetworkConfiguration;
///
/// // The SUBNETS env var is a comma separated list of subnets
/// env::set_var(
///     "SUBNETS",
///     "subnet-0941bb6933678e431,subnet-06581bd8767347cd0",
/// );
///
/// let network_config = envy::from_env::<EcsNetworkConfiguration>().unwrap();
/// ```
#[derive(Debug, Deserialize, Clone)]
pub struct EcsNetworkConfiguration {
    /// The list of subnets for the configuration
    subnets: Vec<String>,
}

impl EcsNetworkConfiguration {
    pub fn new(subnets: Vec<String>) -> Self {
        EcsNetworkConfiguration { subnets }
    }
}

impl From<EcsNetworkConfiguration> for NetworkConfiguration {
    fn from(network_config: EcsNetworkConfiguration) -> Self {
        NetworkConfiguration::builder()
            .awsvpc_configuration(
                AwsVpcConfiguration::builder()
                    .set_subnets(Some(network_config.subnets))
                    .assign_public_ip(AssignPublicIp::Enabled)
                    .build(),
            )
            .build()
    }
}

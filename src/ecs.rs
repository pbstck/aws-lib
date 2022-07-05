use crate::config::get_default_config;
use crate::errors::AwsError;
pub use aws_sdk_ecs::model::{
    AssignPublicIp, AwsVpcConfiguration, KeyValuePair, NetworkConfiguration,
};
use aws_sdk_ecs::model::{ContainerOverride, LaunchType, PropagateTags, TaskOverride};
use aws_sdk_ecs::RetryConfig;
use aws_types::SdkConfig;

/// An Elastic Container Service client
pub struct EcsClient {
    /// The underlying ECS client from the AWS SDK.
    client: aws_sdk_ecs::Client,
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

    /// Launches a task definition
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
        network_config: NetworkConfiguration,
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
            .network_configuration(network_config)
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
}

/// A trait to make data structures convertible into key-value pairs used to override containers environments.
pub trait EcsOverrideEnv {
    /// Creates the vector of key-value pairs.
    ///
    /// Example taken from the `kleanads-build-spawner` project:
    ///
    /// ```
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

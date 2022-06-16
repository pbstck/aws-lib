use aws_types::SdkConfig;

pub async fn get_default_config() -> SdkConfig {
  aws_config::load_from_env().await
}
use aws_config::RetryConfig;

use aws_types::SdkConfig;

use super::errors::{AwsError, DEFAULT_RETRY_COUNT};
use serde_dynamo::aws_sdk_dynamodb_0_13::from_items;

pub struct DynamoDbClient {
    client: aws_sdk_dynamodb::Client,
}

impl DynamoDbClient {
    /// Creates the client with the given configuration and configure the number of retries to [DEFAULT_RETRY_COUNT]
    pub fn new(config: &SdkConfig) -> DynamoDbClient {
        let dynamo_config = aws_sdk_dynamodb::config::Builder::from(config)
            .retry_config(RetryConfig::new().with_max_attempts(DEFAULT_RETRY_COUNT))
            .build();
        let client = aws_sdk_dynamodb::Client::from_conf(dynamo_config);
        DynamoDbClient { client }
    }

    /// Performs a `scan` operation on the `table_name` and deserialize the result.
    pub async fn scan<'a, T>(&self, table_name: &str) -> Result<Vec<T>, AwsError>
    where
        T: serde::Deserialize<'a>,
    {
        let mut first_iteration = true;
        let mut token = None;
        let mut elements = vec![];
        while first_iteration || token.is_some() {
            first_iteration = false;

            match self
                .client
                .scan()
                .table_name(table_name)
                .set_exclusive_start_key(token)
                .send()
                .await
            {
                Ok(result) => {
                    token = result.last_evaluated_key;
                    if let Some(items) = result.items {
                        let element_batch: Vec<T> = match from_items(items) {
                            Ok(element) => element,
                            Err(e) => {
                                return Err(AwsError::new(&format!(
                            "Unable to parse items, error during serialization operation: {}",
                            e
                        )))
                            }
                        };
                        for element in element_batch {
                            elements.push(element);
                        }
                    }
                }
                Err(e) => {
                    return Err(AwsError::new(&format!(
                        "Unable to get items error during scan operation: {}",
                        e
                    )))
                }
            };
        }
        Ok(elements)
    }
}

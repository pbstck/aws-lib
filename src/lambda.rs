use aws_sdk_lambda::error::InvokeAsyncError;
use bytes::Bytes;

use aws_sdk_lambda::model::LogType::Tail;
use aws_sdk_lambda::output::InvokeAsyncOutput;
use aws_sdk_lambda::types::{Blob, SdkError};
use aws_sdk_lambda::RetryConfig;
use aws_smithy_types::base64;
use aws_types::SdkConfig;

pub struct LambdaClient {
    client: aws_sdk_lambda::Client,
}

pub struct LambdaResult {
    base64_logs: String,
    payload: Vec<u8>,
}

impl LambdaResult {
    pub fn get_logs(&self) -> String {
        String::from_utf8(base64::decode(&self.base64_logs).unwrap()).unwrap()
    }
    pub fn get_payload(&self) -> String {
        String::from_utf8(self.payload.clone()).unwrap()
    }
}

impl LambdaClient {
    pub fn new(config: &SdkConfig) -> LambdaClient {
        let lambda_config = aws_sdk_lambda::config::Builder::from(config)
            .retry_config(RetryConfig::new().with_max_attempts(2))
            .build();
        let client = aws_sdk_lambda::Client::from_conf(lambda_config);
        LambdaClient { client }
    }

    pub async fn invoke(&self, function_name: &str, payload: String) -> LambdaResult {
        let res = self
            .client
            .invoke()
            .function_name(function_name)
            .payload(Blob::new(payload))
            .log_type(Tail)
            .send()
            .await
            .unwrap();
        LambdaResult {
            base64_logs: res.log_result.unwrap_or("".to_string()),
            payload: res.payload.unwrap().into_inner(),
        }
    }

    pub async fn invoke_async(
        &self,
        function_name: &str,
        payload: String,
    ) -> Result<InvokeAsyncOutput, SdkError<InvokeAsyncError>> {
        let stream = aws_sdk_lambda::types::ByteStream::from(Bytes::from(payload));
        self.client
            .invoke_async()
            .function_name(function_name)
            .invoke_args(stream)
            .send()
            .await
    }
}

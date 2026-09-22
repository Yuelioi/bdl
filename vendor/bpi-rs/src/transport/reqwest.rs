use tokio::time::Instant;

use reqwest::{Client, RequestBuilder};

use crate::{BpiError, BpiResult};

use super::{RequestMetadata, ResponseMetadata, TransportResponse};

/// endpoint 模块迁移期间使用的基于 reqwest 的 transport 占位类型。
#[derive(Debug, Clone)]
pub struct ReqwestTransport {
    client: Client,
}

impl ReqwestTransport {
    pub fn new(client: Client) -> Self {
        Self { client }
    }

    pub fn client(&self) -> &Client {
        &self.client
    }

    pub async fn send_request_builder(
        builder: RequestBuilder,
        endpoint: &str,
    ) -> BpiResult<TransportResponse> {
        let retry = if super::observer::retry_transient_get()
            && RequestMetadata::from_builder(&builder, endpoint)
                .is_some_and(|metadata| metadata.method == reqwest::Method::GET)
        {
            builder.try_clone()
        } else {
            None
        };
        let result = Self::send_once(builder, endpoint).await;
        let transient = result.as_ref().is_err_and(|error| match error {
            BpiError::Transport { source } => source.is_timeout() || source.is_connect(),
            BpiError::Http { status } | BpiError::HttpStatus { status } => {
                matches!(status, 502 | 503 | 504)
            }
            _ => false,
        });
        if transient && let Some(retry) = retry {
            tokio::time::sleep(std::time::Duration::from_secs(2)).await;
            return Self::send_once(retry, endpoint).await;
        }
        result
    }

    async fn send_once(builder: RequestBuilder, endpoint: &str) -> BpiResult<TransportResponse> {
        let request_metadata = RequestMetadata::from_builder(&builder, endpoint);
        if let Some(metadata) = &request_metadata {
            tracing::info!(
                endpoint = metadata.endpoint.as_str(),
                method = %metadata.method,
                url = metadata.sanitized_url.as_str(),
                "sending Bilibili request"
            );
        } else {
            tracing::info!(endpoint, "sending Bilibili request");
        }

        let mut permit = super::observer::before().await?;
        let start = Instant::now();
        let response = builder.send().await.map_err(BpiError::from)?;
        let status = response.status();
        let headers = response.headers().clone();
        if let Some(permit) = &mut permit {
            permit.observe(status.as_u16(), &headers, &[])?;
        }

        if !status.is_success() {
            tracing::error!(
                endpoint,
                status = status.as_u16(),
                "Bilibili request returned HTTP error"
            );
            return Err(BpiError::http(status.as_u16()));
        }

        let body = response.bytes().await.map_err(BpiError::from)?;
        if let Some(permit) = &mut permit {
            permit.observe(status.as_u16(), &headers, &body)?;
        }
        let duration = start.elapsed();
        tracing::info!(
            endpoint,
            status = status.as_u16(),
            duration_ms = duration.as_millis(),
            "Bilibili request completed"
        );

        Ok(TransportResponse {
            metadata: ResponseMetadata {
                status: status.as_u16(),
                duration,
                api_code: None,
            },
            body,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reqwest_transport_exposes_inner_client() {
        let client = Client::new();
        let transport = ReqwestTransport::new(client);

        let _ = transport.client();
    }
}

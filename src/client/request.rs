use serde::{Serialize, de::DeserializeOwned};
use std::time::Instant;

use crate::{client::VrcClient, error::VrcError};

impl VrcClient {
    pub async fn request_value(
        &self,
        method: reqwest::Method,
        path: &str,
        body: Option<&serde_json::Value>,
    ) -> Result<serde_json::Value, VrcError> {
        let mut request = self.http().request(method.clone(), self.endpoint(path)?);
        if let Some(body) = body {
            request = request.json(body);
        }
        send_json(request, method, path).await
    }

    pub async fn get_json<T>(&self, path: &str) -> Result<T, VrcError>
    where
        T: DeserializeOwned,
    {
        let url = self.endpoint(path)?;
        send_json(self.http().get(url), reqwest::Method::GET, path).await
    }

    pub async fn get_json_with_query<T, Q>(&self, path: &str, query: &Q) -> Result<T, VrcError>
    where
        T: DeserializeOwned,
        Q: Serialize + ?Sized,
    {
        let mut url = self.endpoint(path)?;
        let query = serde_urlencoded::to_string(query)?;

        if !query.is_empty() {
            url.set_query(Some(&query));
        }

        send_json(self.http().get(url), reqwest::Method::GET, path).await
    }

    pub async fn post_json<T, B>(&self, path: &str, body: &B) -> Result<T, VrcError>
    where
        T: DeserializeOwned,
        B: Serialize + ?Sized,
    {
        let url = self.endpoint(path)?;
        send_json(
            self.http().post(url).json(body),
            reqwest::Method::POST,
            path,
        )
        .await
    }

    pub async fn put_json<T, B>(&self, path: &str, body: &B) -> Result<T, VrcError>
    where
        T: DeserializeOwned,
        B: Serialize + ?Sized,
    {
        let url = self.endpoint(path)?;
        send_json(self.http().put(url).json(body), reqwest::Method::PUT, path).await
    }

    pub async fn put_empty<B>(&self, path: &str, body: &B) -> Result<(), VrcError>
    where
        B: Serialize + ?Sized,
    {
        let started = Instant::now();
        tracing::debug!(method = "PUT", path, "HTTP request");
        let response = self
            .http()
            .put(self.endpoint(path)?)
            .json(body)
            .send()
            .await
            .map_err(VrcError::Network)?;
        let status = response.status();
        tracing::debug!(
            method = "PUT",
            path,
            %status,
            elapsed_ms = started.elapsed().as_millis(),
            "HTTP response"
        );
        if status.is_success() {
            Ok(())
        } else {
            let body = response.text().await.unwrap_or_default();
            Err(VrcError::Api { status, body })
        }
    }

    pub async fn patch_json<T, B>(&self, path: &str, body: &B) -> Result<T, VrcError>
    where
        T: DeserializeOwned,
        B: Serialize + ?Sized,
    {
        let url = self.endpoint(path)?;
        send_json(
            self.http().patch(url).json(body),
            reqwest::Method::PATCH,
            path,
        )
        .await
    }

    pub async fn delete_json<T>(&self, path: &str) -> Result<T, VrcError>
    where
        T: DeserializeOwned,
    {
        let url = self.endpoint(path)?;
        send_json(self.http().delete(url), reqwest::Method::DELETE, path).await
    }

    pub async fn delete_json_with_query<T, Q>(&self, path: &str, query: &Q) -> Result<T, VrcError>
    where
        T: DeserializeOwned,
        Q: Serialize + ?Sized,
    {
        let mut url = self.endpoint(path)?;
        let query = serde_urlencoded::to_string(query)?;
        if !query.is_empty() {
            url.set_query(Some(&query));
        }
        send_json(self.http().delete(url), reqwest::Method::DELETE, path).await
    }

    pub async fn get_text(&self, path: &str) -> Result<String, VrcError> {
        let started = Instant::now();
        tracing::debug!(method = "GET", path, "HTTP request");
        let response = self
            .http()
            .get(self.endpoint(path)?)
            .send()
            .await
            .map_err(|error| {
                tracing::error!(method = "GET", path, %error, "HTTP request failed");
                VrcError::Network(error)
            })?;
        let status = response.status();
        tracing::debug!(
            method = "GET",
            path,
            %status,
            elapsed_ms = started.elapsed().as_millis(),
            "HTTP response"
        );
        let body = response
            .text()
            .await
            .map_err(|error| VrcError::Decode(error.to_string()))?;
        if status.is_success() {
            Ok(body)
        } else {
            Err(VrcError::Api { status, body })
        }
    }
}

async fn send_json<T>(
    request: reqwest::RequestBuilder,
    method: reqwest::Method,
    path: &str,
) -> Result<T, VrcError>
where
    T: DeserializeOwned,
{
    let started = Instant::now();
    tracing::debug!(%method, path, "HTTP request");
    let response = request.send().await.map_err(|error| {
        tracing::error!(%method, path, %error, "HTTP request failed");
        VrcError::Network(error)
    })?;
    let status = response.status();
    tracing::debug!(
        %method,
        path,
        %status,
        elapsed_ms = started.elapsed().as_millis(),
        "HTTP response"
    );

    if !status.is_success() {
        let body = response.text().await.unwrap_or_else(|err| err.to_string());
        tracing::warn!(%method, path, %status, "HTTP API error");
        return Err(VrcError::Api { status, body });
    }

    let body = response
        .text()
        .await
        .map_err(|error| VrcError::Decode(error.to_string()))?;

    serde_json::from_str::<T>(&body).map_err(|error| {
        tracing::error!(%method, path, %error, "HTTP response decode failed");
        VrcError::Decode(format!(
            "{}; body: {}",
            error,
            body.chars().take(1000).collect::<String>()
        ))
    })
}

#[cfg(test)]
mod tests {
    use serde::Serialize;

    use crate::client::{VrcClient, config::ClientConfig};

    #[derive(Serialize)]
    struct Query<'a> {
        #[serde(rename = "n")]
        limit: u8,
        search: &'a str,
    }

    #[test]
    fn endpoint_joins_relative_api_paths() {
        let client = VrcClient::new(
            reqwest::Client::new(),
            ClientConfig::new(
                url::Url::parse("https://example.test/api/1/").unwrap(),
                "test",
            ),
        );

        assert_eq!(
            client.endpoint("/worlds/wrld_1").unwrap().as_str(),
            "https://example.test/api/1/worlds/wrld_1"
        );
    }

    #[test]
    fn query_types_use_api_parameter_names() {
        let encoded = serde_urlencoded::to_string(Query {
            limit: 50,
            search: "hello world",
        })
        .unwrap();

        assert_eq!(encoded, "n=50&search=hello+world");
    }
}

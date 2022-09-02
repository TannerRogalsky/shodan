use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct Client {
    token: String,
    model_version: String,
    http: reqwest::Client,
}

impl Client {
    pub fn new(token: impl Into<String>, version: impl Into<String>) -> Self {
        let http = reqwest::Client::new();
        Self {
            token: token.into(),
            model_version: version.into(),
            http,
        }
    }

    pub async fn new_prediction(&self, prompt: impl AsRef<str>) -> reqwest::Result<Vec<url::Url>> {
        let body = CreatePrediction {
            version: &self.model_version,
            input: CreatePredictionInput {
                prompt: prompt.as_ref().into(),
                ..Default::default()
            },
        };
        let resp = self
            .http
            .post("https://api.replicate.com/v1/predictions")
            .header(
                reqwest::header::AUTHORIZATION,
                format!("Token {}", self.token),
            )
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .json(&body)
            .send()
            .await?;

        let mut status: StatusResponse = resp.json().await?;
        while status.output.is_none() {
            futures_timer::Delay::new(std::time::Duration::from_secs(1)).await;
            status = self.get_prediction(status.id).await?;
        }

        Ok(status.output.unwrap())
    }

    async fn get_prediction(&self, id: impl AsRef<str>) -> reqwest::Result<StatusResponse> {
        let resp = self
            .http
            .get(format!(
                "https://api.replicate.com/v1/predictions/{}",
                id.as_ref()
            ))
            .header(
                reqwest::header::AUTHORIZATION,
                format!("Token {}", self.token),
            )
            .send()
            .await?;
        let status = resp.json().await?;
        Ok(status)
    }
}

// https://replicate.com/stability-ai/stable-diffusion
#[derive(Debug, Serialize, Deserialize)]
struct CreatePredictionInput<'a> {
    prompt: std::borrow::Cow<'a, str>,
    width: u32,
    height: u32,
    prompt_strength: f32,
    num_outputs: u32,
    num_inference_steps: u32,
    guidance_scale: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    seed: Option<u32>,
    // mask
    // init_image
}

impl Default for CreatePredictionInput<'_> {
    fn default() -> Self {
        Self {
            prompt: Default::default(),
            width: 512,
            height: 512,
            prompt_strength: 0.8,
            num_outputs: 1,
            num_inference_steps: 50,
            guidance_scale: 7.5,
            seed: None,
        }
    }
}

#[derive(Debug, Serialize)]
struct CreatePrediction<'a> {
    version: &'a str,
    input: CreatePredictionInput<'a>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
enum Status {
    Starting,
    Processing,
    Succeeded,
    Failed,
    Canceled,
}

#[derive(Debug, Deserialize)]
#[allow(unused)]
struct StatusUrls {
    get: url::Url,
    cancel: url::Url,
}

#[derive(Debug, Deserialize)]
#[allow(unused)]
struct StatusResult {
    id: String,
    version: String,
    urls: StatusUrls,
    created_at: String,
    completed_at: String,
    source: String,
    status: Status,
}

#[derive(Debug, Deserialize)]
#[allow(unused)]
struct StatusResponse {
    id: String,
    version: String,
    urls: StatusUrls,
    created_at: String,
    completed_at: String,
    status: Status,
    input: CreatePredictionInput<'static>,
    output: Option<Vec<url::Url>>,
    error: Option<String>,
    metrics: serde_json::Map<String, serde_json::Value>,
    logs: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn it_works() {
        dotenv::dotenv().unwrap();
        let token = std::env::var("STABDIFF_TOKEN").unwrap();
        let version = "a9758cbfbd5f3c2094457d996681af52552901775aa2d6dd0b17fd15df959bef";
        let client = Client::new(token, version);
        let _r = client.new_prediction("multicolor hyperspace").await;
    }
}

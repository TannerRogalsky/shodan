#[derive(Debug, Clone)]
pub struct Dalle {
    client: reqwest::Client,
    url: reqwest::Url,
}

impl Dalle {
    pub async fn new<U: reqwest::IntoUrl>(url: U) -> Result<Self, Box<dyn std::error::Error>> {
        let url = url.into_url()?;
        let client = reqwest::Client::new();
        let hc_resp = client.get(url.clone()).send().await?;
        if hc_resp.status() != reqwest::StatusCode::OK {
            return Err(format!(
                "Couldn't connect to Dalle backend {}: response {}",
                url.as_str(),
                hc_resp.status()
            )
            .into());
        } else {
            let url = url.join("/dalle")?;
            Ok(Self { client, url })
        }
    }

    pub async fn generate(&self, prompt: &str, count: usize) -> reqwest::Result<GenerateResponse> {
        #[derive(serde::Serialize)]
        struct GenerateBody<'a> {
            text: &'a str,
            num_images: usize,
        }

        self.client
            .post(self.url.clone())
            .json(&GenerateBody {
                text: prompt,
                num_images: count,
            })
            .send()
            .await?
            .json()
            .await
    }
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct GenerateResponse {
    #[serde(rename = "generatedImgs")]
    pub generated_imgs: Vec<String>,
    #[serde(rename = "generatedImgsFormat")]
    pub generated_imgs_format: String,
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}

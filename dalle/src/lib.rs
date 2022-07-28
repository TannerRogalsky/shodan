use thiserror::Error;

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

    pub async fn generate(
        &self,
        prompt: &str,
        count: usize,
    ) -> Result<GenerateResponse, GenerateError> {
        #[derive(serde::Serialize)]
        struct GenerateBody<'a> {
            text: &'a str,
            num_images: usize,
        }

        Ok(self
            .client
            .post(self.url.clone())
            .json(&GenerateBody {
                text: prompt,
                num_images: count,
            })
            .send()
            .await?
            .json::<Intermediate>()
            .await?
            .try_into()?)
    }
}

#[derive(Error, Debug)]
pub enum GenerateError {
    #[error("Request error: {0}")]
    Reqwest(#[from] reqwest::Error),
    #[error("Base64 decode error: {0}")]
    DecodeError(#[from] base64::DecodeError),
}

#[derive(Debug, Clone, serde::Deserialize)]
struct Intermediate {
    #[serde(rename = "generatedImgs")]
    generated_imgs: Vec<String>,
    #[serde(rename = "generatedImgsFormat")]
    generated_imgs_format: String,
}

impl TryFrom<Intermediate> for GenerateResponse {
    type Error = base64::DecodeError;

    fn try_from(value: Intermediate) -> Result<Self, Self::Error> {
        Ok(Self {
            generated_imgs: value
                .generated_imgs
                .into_iter()
                .map(|v| base64::decode(v))
                .collect::<Result<_, _>>()?,
            generated_imgs_format: value.generated_imgs_format,
        })
    }
}

#[derive(Debug, Clone)]
pub struct GenerateResponse {
    pub generated_imgs: Vec<Vec<u8>>,
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

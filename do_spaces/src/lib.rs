use std::str::FromStr;
use thiserror::Error;

// this is a macro instead of a raw literal so that i can use concat!
macro_rules! endpoint {
    () => {
        "nyc3.digitaloceanspaces.com"
    };
}
const BUCKET: &'static str = "wutangtan";

#[derive(Debug, Error)]
pub enum Error {
    #[error("AWS error: {0}")]
    AWS(#[from] aws_sdk_s3::types::SdkError<aws_sdk_s3::error::PutObjectError>),
    #[error("URI parse error: {0}")]
    URI(#[from] http::uri::InvalidUri),
}

#[derive(Debug, Clone)]
pub struct Client {
    s3: aws_sdk_s3::Client,
}

impl Client {
    pub fn new<S: Into<String>>(secret_access_key: S) -> Self {
        let config = aws_types::sdk_config::SdkConfig::builder()
            .region(aws_types::region::Region::new("nyc3"))
            .endpoint_resolver(aws_smithy_http::endpoint::Endpoint::immutable(
                http::Uri::from_static(concat!("https://", endpoint!())),
            ))
            .credentials_provider(aws_types::credentials::SharedCredentialsProvider::new(
                aws_types::Credentials::new(
                    "DO00BQFTGHJ6QZZAP3CB",
                    secret_access_key,
                    None,
                    None,
                    "spaces_cred_provider",
                ),
            ))
            .build();

        let s3 = aws_sdk_s3::Client::new(&config);
        Self { s3 }
    }

    pub async fn put_jpeg<S: Into<String>>(
        &self,
        key: S,
        data: Vec<u8>,
    ) -> Result<http::Uri, Error> {
        let key = key.into();
        let url = http::Uri::from_str(&format!("http://{}.{}/{}", BUCKET, endpoint!(), key))?;
        self.s3
            .put_object()
            .bucket(BUCKET)
            .key(key)
            .content_type("image/jpeg")
            .acl(aws_sdk_s3::model::ObjectCannedAcl::PublicRead)
            .body(aws_sdk_s3::types::ByteStream::from(data))
            .send()
            .await?;
        Ok(url)
    }

    pub async fn put_text<S: Into<String>>(
        &self,
        key: S,
        data: String,
    ) -> Result<http::Uri, Error> {
        let key = key.into();
        let url = http::Uri::from_str(&format!("http://{}.{}/{}", BUCKET, endpoint!(), key))?;
        self.s3
            .put_object()
            .bucket(BUCKET)
            .key(key)
            .content_type("text/plain")
            .acl(aws_sdk_s3::model::ObjectCannedAcl::PublicRead)
            .body(aws_sdk_s3::types::ByteStream::new(data.into()))
            .send()
            .await?;
        Ok(url)
    }
}

#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn it_works() {
        assert!(true)
    }
}

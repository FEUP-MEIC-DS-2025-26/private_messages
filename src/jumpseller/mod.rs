use anyhow::anyhow;

use crate::JumpSellerCredentials;

pub struct Client {
    login: String,
    token: String,
    client: reqwest::Client,
}

impl From<JumpSellerCredentials> for Client {
    fn from(value: JumpSellerCredentials) -> Self {
        Self::new(value.login, value.token)
    }
}

#[derive(serde::Deserialize, Debug)]
pub struct Product {
    id: u64,
    name: String,
}

#[derive(serde::Deserialize)]
struct ProductWrapper {
    product: Product,
}

impl Client {
    pub fn new(login: String, token: String) -> Self {
        Self {
            login,
            token,
            client: reqwest::Client::new(),
        }
    }

    pub async fn get_product(&self, id: u64) -> anyhow::Result<Product> {
        let response = self.client
            .get(format!("https://api.jumpseller.com/v1/products/{id}.json"))
            .basic_auth(&self.login, Some(&self.token))
            .send()
            .await?;

        let status = response.status();
        let body = response.text().await?;

        if status != 200 {
            let json: serde_json::Value = serde_json::from_str(&body)?;
            let err_str = json
                .get("error")
                .and_then(|v| v.as_str())
                .unwrap_or("Request failed")
                ;
            return Err(anyhow!(err_str.to_owned()));
        }

        let product = serde_json::from_str::<ProductWrapper>(&body)?.product;
        Ok(product)
    }
}

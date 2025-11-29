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

#[derive(Debug)]
pub enum JumpSellerErr {
    /// error occured before getting response (e.g. jumpseller unavailable)
    RequestErr(reqwest::Error),

    /// error occured when processing response (e.g. product not found when calling get_product())
    ResponseErr(anyhow::Error),
}

// impl std::fmt::Display for JumpSellerErr {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         write!(f, "{:?}", self)
//     }
// }

// impl std::error::Error for JumpSellerErr {}

impl Client {
    pub fn new(login: String, token: String) -> Self {
        Self {
            login,
            token,
            client: reqwest::Client::new(),
        }
    }

    pub async fn get_product(&self, id: u64) -> anyhow::Result<Product, JumpSellerErr> {
        let response = self.client
            .get(format!("https://api.jumpseller.com/v1/products/{id}.json"))
            .basic_auth(&self.login, Some(&self.token))
            .send()
            .await
            .or_else(|err| Err(JumpSellerErr::RequestErr(err)))
            ?;

        let status = response.status();
        let body = response.text().await.or_else(|err| Err(JumpSellerErr::ResponseErr(err.into())))?;

        if status != 200 {
            let json: serde_json::Value = serde_json::from_str(&body).or_else(|err| Err(JumpSellerErr::ResponseErr(err.into())))?;
            let err_str = json
                .get("message")
                .and_then(|v| v.as_str())
                .unwrap_or("Request failed")
                ;
            return Err(JumpSellerErr::ResponseErr(anyhow!(err_str.to_owned())));
        }

        let product = serde_json::from_str::<ProductWrapper>(&body)
            .or_else(|err| Err(JumpSellerErr::ResponseErr(err.into())))?
            .product;

        Ok(product)
    }
}

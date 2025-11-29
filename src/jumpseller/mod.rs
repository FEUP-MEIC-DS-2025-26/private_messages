use anyhow::anyhow;

use crate::JumpSellerCredentials;

pub enum Client {
    Dummy,
    Client {
        login: String,
        token: String,
        client: reqwest::Client,
    },
}

pub struct ClientGuard<'a> {
    login: &'a String,
    token: &'a String,
    client: &'a reqwest::Client,
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

#[derive(Debug, thiserror::Error)]
pub enum JumpSellerErr {
    #[error("Tried to call method on dummy client")]
    IsDummy,

    #[error("Failed to request from JumpSeller: {0}")]
    RequestErr(reqwest::Error),

    #[error("Response from JumpSeller: {0}")]
    ResponseErr(anyhow::Error, Option<reqwest::StatusCode>),
}

// impl std::fmt::Display for JumpSellerErr {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         write!(f, "{:?}", self)
//     }
// }

// impl std::error::Error for JumpSellerErr {}

impl Client {
    pub fn new(login: String, token: String) -> Self {
        Self::Client {
            login,
            token,
            client: reqwest::Client::new(),
        }
    }

    pub const fn dummy() -> Self {
        Self::Dummy
    }

    pub fn get_guard<'a>(&'a self) -> Result<ClientGuard<'a>, JumpSellerErr> {
        match self {
            Client::Dummy => return Err(JumpSellerErr::IsDummy),
            Client::Client {
                login,
                token,
                client,
            } => Ok(ClientGuard {
                login,
                token,
                client,
            }),
        }
    }

    pub async fn get_product(&self, id: u64) -> anyhow::Result<Product, JumpSellerErr> {
        let this = self.get_guard()?;

        let response = this
            .client
            .get(format!("https://api.jumpseller.com/v1/products/{id}.json"))
            .basic_auth(&this.login, Some(&this.token))
            .send()
            .await
            .or_else(|err| Err(JumpSellerErr::RequestErr(err)))?;

        let status = response.status();
        let body = response
            .text()
            .await
            .or_else(|err| Err(JumpSellerErr::ResponseErr(err.into(), None)))?;

        if status != 200 {
            let json: serde_json::Value = serde_json::from_str(&body)
                .or_else(|err| Err(JumpSellerErr::ResponseErr(err.into(), None)))?;
            let err_str = json
                .get("message")
                .and_then(|v| v.as_str())
                .unwrap_or("Request failed");
            return Err(JumpSellerErr::ResponseErr(
                anyhow!(err_str.to_owned()),
                Some(status),
            ));
        }

        let product = serde_json::from_str::<ProductWrapper>(&body)
            .or_else(|err| Err(JumpSellerErr::ResponseErr(err.into(), None)))?
            .product;

        Ok(product)
    }
}

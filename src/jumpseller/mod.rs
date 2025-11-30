use actix_web::http::StatusCode as ActixStatusCode;
use anyhow::anyhow;

use crate::{JumpSellerCredentials, database::sqlite::UserProfile};

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

impl actix_web::ResponseError for JumpSellerErr {
    fn status_code(&self) -> ActixStatusCode {
        match self {
            JumpSellerErr::IsDummy => ActixStatusCode::INTERNAL_SERVER_ERROR,
            JumpSellerErr::RequestErr(_) => ActixStatusCode::SERVICE_UNAVAILABLE,
            JumpSellerErr::ResponseErr(_, status_code) => match status_code {
                Some(code) => ActixStatusCode::from_u16(code.as_u16())
                    .unwrap_or(ActixStatusCode::INTERNAL_SERVER_ERROR),
                None => ActixStatusCode::INTERNAL_SERVER_ERROR,
            },
        }
    }
}

#[derive(serde::Deserialize, Debug)]
pub struct UserRetrieval {
    fullname: String,
    email: String,
}

trait CustomerExtractor {}
impl CustomerExtractor for UserRetrieval {}

#[derive(serde::Deserialize, Debug)]
struct CustomerWrapper<T: CustomerExtractor> {
    customer: T,
}

#[derive(serde::Deserialize, Debug)]
pub struct Product {
    pub id: i64,
    pub name: String,
}

trait ProductExtractor {}
impl ProductExtractor for Product {}

#[derive(serde::Deserialize)]
struct ProductWrapper<T: ProductExtractor> {
    product: T,
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

    fn get_guard(&self) -> Result<ClientGuard<'_>, JumpSellerErr> {
        match self {
            Client::Dummy => Err(JumpSellerErr::IsDummy),
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

    pub async fn get_product(&self, id: i64) -> Result<Product, JumpSellerErr> {
        let this = self.get_guard()?;

        let response = this
            .client
            .get(format!("https://api.jumpseller.com/v1/products/{id}.json"))
            .basic_auth(this.login, Some(&this.token))
            .send()
            .await
            .map_err(JumpSellerErr::RequestErr)?;

        let status = response.status();
        let body = response
            .text()
            .await
            .map_err(|err| JumpSellerErr::ResponseErr(err.into(), None))?;

        if status != 200 {
            let json: serde_json::Value = serde_json::from_str(&body)
                .map_err(|err| JumpSellerErr::ResponseErr(err.into(), None))?;
            let err_str = json
                .get("message")
                .and_then(|v| v.as_str())
                .unwrap_or("Request failed");
            return Err(JumpSellerErr::ResponseErr(
                anyhow!(err_str.to_owned()),
                Some(status),
            ));
        }

        let product = serde_json::from_str::<ProductWrapper<Product>>(&body)
            .map_err(|err| JumpSellerErr::ResponseErr(err.into(), None))?
            .product;

        Ok(product)
    }

    pub async fn get_user(&self, user_id: i64) -> Result<UserProfile, JumpSellerErr> {
        let this = self.get_guard()?;

        let response = this
            .client
            .get(format!(
                "https://api.jumpseller.com/v1/customers/{user_id}.json"
            ))
            .basic_auth(this.login, Some(&this.token))
            .send()
            .await
            .map_err(JumpSellerErr::RequestErr)?;

        let status = response.status();
        let body = response
            .text()
            .await
            .map_err(|e| JumpSellerErr::ResponseErr(e.into(), None))?;

        if status != 200 {
            let json: serde_json::Value = serde_json::from_str(&body)
                .map_err(|err| JumpSellerErr::ResponseErr(err.into(), None))?;
            let err_str = json
                .get("message")
                .and_then(|v| v.as_str())
                .unwrap_or("Request failed");
            return Err(JumpSellerErr::ResponseErr(
                anyhow!(err_str.to_owned()),
                Some(status),
            ));
        }
        let user = serde_json::from_str::<CustomerWrapper<UserRetrieval>>(&body)
            .map_err(|err| JumpSellerErr::ResponseErr(err.into(), None))?
            .customer;

        let username = user
            .email
            .split('@')
            .next()
            .unwrap_or("anonymous")
            .to_string();

        Ok(UserProfile::new(user_id, username, user.fullname))
    }
}

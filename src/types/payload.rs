use super::data::User;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct EmailSignUpBody {
    pub name: String,
    pub email: String,
    pub password: String,
    pub image: Option<String>,
    #[serde(rename = "rememberMe")]
    pub remember_me: Option<bool>,
    #[serde(rename = "callbackURL")]
    pub callback_url: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct EmailSignInResponse {
    pub token: Option<String>,
    pub user: Option<User>,
}

#[derive(Serialize, Deserialize)]
pub struct EmailSignInBody {
    pub email: String,
    pub password: String,
    #[serde(rename = "rememberMe")]
    pub remember_me: Option<bool>,
    #[serde(rename = "callbackURL")]
    pub callback_url: Option<String>,
}

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct EmailSignUpBody {
    pub name: String,
    pub email: String,
    pub password: String,
    pub image: Option<String>,
    #[serde(rename = "rememberMe")]
    pub remember_me: Option<bool>,
    #[serde(rename = "callbackUrl")]
    pub callback_url: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct EmailSignInBody {
    pub email: String,
    pub password: String,
    #[serde(rename = "rememberMe")]
    pub remember_me: Option<bool>,
    #[serde(rename = "callbackUrl")]
    pub callback_url: Option<String>,
}

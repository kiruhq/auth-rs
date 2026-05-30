use crate::core::entity::User;

pub(crate) enum CreateUserError {
    Stub,
}

pub(crate) enum GetUserError {
    Stub,
}

pub(crate) struct CreateUser {
    pub id: String,
    pub name: String,
    pub email: String,
    pub image: Option<String>,
}

#[async_trait::async_trait]
pub(crate) trait UserStore: Send {
    async fn create_user(&mut self, user: CreateUser) -> Result<User, CreateUserError>;
    async fn get_user_by_id(&mut self, id: &str) -> Result<Option<User>, GetUserError>;
    async fn get_user_by_email(&mut self, email: &str) -> Result<Option<User>, GetUserError>;
}

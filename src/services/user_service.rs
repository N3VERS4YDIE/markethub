use crate::{error::AppError, models::user::PublicUser, repositories::UserRepository};
use uuid::Uuid;

#[derive(Clone)]
pub struct UserService {
    users: UserRepository,
}

impl UserService {
    pub fn new(users: UserRepository) -> Self {
        Self { users }
    }

    pub async fn get_profile(&self, user_id: Uuid) -> crate::Result<PublicUser> {
        let user = self
            .users
            .find_by_id(user_id)
            .await?
            .ok_or_else(|| AppError::NotFound("User not found".into()))?;

        Ok(user.into())
    }
}

use validator::Validate;

use crate::{
    error::AppError,
    models::permission::Permission,
    models::store::{CreateStoreRequest, MemberRole, Store, StoreMember},
    repositories::{MemberRepository, StoreRepository},
};
use uuid::Uuid;

#[derive(Clone)]
pub struct StoreService {
    stores: StoreRepository,
    members: MemberRepository,
}

impl StoreService {
    pub fn new(stores: StoreRepository, members: MemberRepository) -> Self {
        Self { stores, members }
    }

    pub async fn create_store(
        &self,
        owner_id: Uuid,
        payload: CreateStoreRequest,
    ) -> crate::Result<Store> {
        payload
            .validate()
            .map_err(|err| AppError::Validation(err.to_string()))?;

        if self.stores.find_by_slug(&payload.slug).await?.is_some() {
            return Err(AppError::Conflict("Slug already in use".into()));
        }

        let store = self.stores.create(owner_id, &payload).await?;

        // Ensure owner is registered as store member
        self.members
            .add_member(
                store.id,
                owner_id,
                MemberRole::Owner,
                Permission::all(),
                Some(owner_id),
            )
            .await?;

        Ok(store)
    }

    pub async fn list_public(&self, limit: i64, offset: i64) -> crate::Result<Vec<Store>> {
        self.stores.list_public(limit, offset).await
    }

    pub async fn get_store(&self, store_id: Uuid) -> crate::Result<Store> {
        self.stores
            .find_by_id(store_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Store not found".into()))
    }

    pub async fn list_members(&self, store_id: Uuid) -> crate::Result<Vec<StoreMember>> {
        self.members.list_members(store_id).await
    }
}

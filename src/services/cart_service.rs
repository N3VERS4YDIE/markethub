use validator::Validate;

use crate::{
    error::AppError,
    models::order::{AddCartItemRequest, CartItem, CartItemDetail},
    repositories::{CartRepository, ProductRepository},
};
use uuid::Uuid;

#[derive(Clone)]
pub struct CartService {
    carts: CartRepository,
    products: ProductRepository,
}

impl CartService {
    pub fn new(carts: CartRepository, products: ProductRepository) -> Self {
        Self { carts, products }
    }

    pub async fn add_item(
        &self,
        user_id: Uuid,
        payload: AddCartItemRequest,
    ) -> crate::Result<CartItem> {
        payload
            .validate()
            .map_err(|err| AppError::Validation(err.to_string()))?;

        let product = self
            .products
            .find_by_id(payload.product_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Product not found".into()))?;

        if !product.is_active {
            return Err(AppError::BadRequest("Product is inactive".into()));
        }

        if product.stock_quantity < payload.quantity {
            return Err(AppError::Conflict("Insufficient stock".into()));
        }

        self.carts
            .upsert_item(user_id, payload.product_id, payload.quantity)
            .await
    }

    pub async fn list_items(&self, user_id: Uuid) -> crate::Result<Vec<CartItemDetail>> {
        self.carts.list_with_products(user_id).await
    }

    pub async fn remove_item(&self, user_id: Uuid, product_id: Uuid) -> crate::Result<()> {
        self.carts.remove_item(user_id, product_id).await
    }

    pub async fn clear(&self, user_id: Uuid) -> crate::Result<()> {
        self.carts.clear_user(user_id).await
    }
}

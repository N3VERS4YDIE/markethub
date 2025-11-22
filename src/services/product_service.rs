use rust_decimal::Decimal;
use validator::Validate;

use crate::{
    error::AppError,
    models::product::{CreateProductRequest, Product, UpdateProductRequest},
    repositories::{ProductRepository, StoreRepository},
};
use uuid::Uuid;

#[derive(Clone)]
pub struct ProductService {
    products: ProductRepository,
    stores: StoreRepository,
}

impl ProductService {
    pub fn new(products: ProductRepository, stores: StoreRepository) -> Self {
        Self { products, stores }
    }

    pub async fn create_product(&self, payload: CreateProductRequest) -> crate::Result<Product> {
        payload
            .validate()
            .map_err(|err| AppError::Validation(err.to_string()))?;

        self.ensure_store_exists(payload.store_id).await?;
        let price = decimal_from_f64(payload.price)?;

        self.products
            .create(
                payload.store_id,
                &payload.sku,
                &payload.name,
                payload.description.as_deref(),
                price,
                payload.stock_quantity,
                payload.category.as_deref(),
            )
            .await
    }

    pub async fn list_by_store(
        &self,
        store_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> crate::Result<Vec<Product>> {
        self.ensure_store_exists(store_id).await?;
        self.products.list_by_store(store_id, limit, offset).await
    }

    pub async fn update_product(
        &self,
        product_id: Uuid,
        payload: UpdateProductRequest,
    ) -> crate::Result<Product> {
        payload
            .validate()
            .map_err(|err| AppError::Validation(err.to_string()))?;

        let mut product = self
            .products
            .find_by_id(product_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Product not found".into()))?;

        if let Some(name) = payload.name {
            product.name = name;
        }
        if let Some(desc) = payload.description {
            product.description = Some(desc);
        }
        if let Some(price) = payload.price {
            product.price = decimal_from_f64(price)?;
        }
        if let Some(stock) = payload.stock_quantity {
            product.stock_quantity = stock;
        }
        if let Some(category) = payload.category {
            product.category = Some(category);
        }
        if let Some(is_active) = payload.is_active {
            product.is_active = is_active;
        }

        // Persist changes
        let updated = self.products.save(&product).await?;

        Ok(updated)
    }

    pub async fn get_product(&self, product_id: Uuid) -> crate::Result<Product> {
        self.products
            .find_by_id(product_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Product not found".into()))
    }

    async fn ensure_store_exists(&self, store_id: Uuid) -> crate::Result<()> {
        if self.stores.find_by_id(store_id).await?.is_none() {
            return Err(AppError::NotFound("Store not found".into()));
        }
        Ok(())
    }
}

fn decimal_from_f64(value: f64) -> crate::Result<Decimal> {
    Decimal::from_f64_retain(value)
        .ok_or_else(|| AppError::Validation("Invalid price value".into()))
}

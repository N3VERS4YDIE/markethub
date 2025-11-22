use chrono::Utc;
use rust_decimal::Decimal;
use serde_json::Value;
use uuid::Uuid;
use validator::Validate;

use crate::{
    error::AppError,
    models::order::{CartItemDetail, CheckoutRequest, CheckoutSummary, Order, PaymentStatus},
    repositories::{CartRepository, OrderRepository, ProductRepository},
};

#[derive(Clone)]
pub struct OrderService {
    orders: OrderRepository,
    products: ProductRepository,
    carts: CartRepository,
}

impl OrderService {
    pub fn new(
        orders: OrderRepository,
        products: ProductRepository,
        carts: CartRepository,
    ) -> Self {
        Self {
            orders,
            products,
            carts,
        }
    }

    pub async fn checkout(
        &self,
        user_id: Uuid,
        payload: CheckoutRequest,
    ) -> crate::Result<CheckoutSummary> {
        payload
            .validate()
            .map_err(|err| AppError::Validation(err.to_string()))?;

        let items = self.carts.list_with_products(user_id).await?;
        if items.is_empty() {
            return Err(AppError::BadRequest("Cart is empty".into()));
        }

        let calculations = self.prepare_calculations(items, payload.shipping_address.clone());
        let group_total = calculations
            .iter()
            .fold(Decimal::ZERO, |acc, calc| acc + calc.total_amount);

        let mut tx = self.orders.pool().begin().await?;
        let group_number = format!("GRP-{}", short_id());
        let order_group = self
            .orders
            .create_group(
                &mut tx,
                user_id,
                &group_number,
                group_total,
                PaymentStatus::Pending,
            )
            .await?;

        let mut created_orders: Vec<Order> = Vec::new();
        for calc in &calculations {
            let order_number = format!("ORD-{}", short_id());
            let order = self
                .orders
                .create_order(
                    &mut tx,
                    order_group.id,
                    user_id,
                    calc.store_id,
                    &order_number,
                    calc.subtotal,
                    calc.tax,
                    calc.discount,
                    calc.shipping_cost,
                    calc.total_amount,
                    &calc.shipping_address,
                )
                .await?;

            for line in &calc.items {
                let line_subtotal = line.unit_price * Decimal::from(line.quantity);
                self.orders
                    .create_order_item(
                        &mut tx,
                        order.id,
                        line.product_id,
                        line.quantity,
                        line.unit_price,
                        line_subtotal,
                    )
                    .await?;

                self.products
                    .decrement_stock_in_tx(&mut tx, line.product_id, line.quantity)
                    .await?;
            }

            created_orders.push(order);
        }

        tx.commit().await?;
        self.carts.clear_user(user_id).await?;

        Ok(CheckoutSummary {
            order_group,
            orders: created_orders,
        })
    }

    pub async fn list_orders(
        &self,
        user_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> crate::Result<Vec<Order>> {
        self.orders
            .list_orders_for_user(user_id, limit, offset)
            .await
    }

    fn prepare_calculations(
        &self,
        grouped_items: Vec<CartItemDetail>,
        shipping_address: Value,
    ) -> Vec<StoreCalculation> {
        let grouped = CartItemDetail::group_by_store(&grouped_items);
        grouped
            .into_iter()
            .map(|(store_id, items)| {
                let subtotal = items.iter().fold(Decimal::ZERO, |acc, item| {
                    acc + item.unit_price * Decimal::from(item.quantity)
                });
                let tax = Decimal::ZERO;
                let discount = Decimal::ZERO;
                let shipping_cost = Decimal::ZERO;
                let total_amount = subtotal + tax + shipping_cost - discount;

                StoreCalculation {
                    store_id,
                    items,
                    subtotal,
                    tax,
                    discount,
                    shipping_cost,
                    total_amount,
                    shipping_address: shipping_address.clone(),
                }
            })
            .collect()
    }
}

struct StoreCalculation {
    store_id: Uuid,
    items: Vec<CartItemDetail>,
    subtotal: Decimal,
    tax: Decimal,
    discount: Decimal,
    shipping_cost: Decimal,
    total_amount: Decimal,
    shipping_address: Value,
}

fn short_id() -> String {
    let now = Utc::now().timestamp_millis();
    format!("{:x}", now)
}

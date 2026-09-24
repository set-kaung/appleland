use rust_decimal::Decimal;
use rust_decimal::prelude::FromPrimitive;
use sqlx::{PgPool, Postgres, Transaction};
use std::collections::HashSet;
use std::ops::Mul;
use uuid::Uuid;

use super::model::{
    AdminPlaygroundPurchase, AdminPlaygroundPurchasesQuery, PlaygroundCheckoutDataAdmin,
    UserPlaygroundAccount, UserPlaygroundPurchase, UserPlaygroundPurchaseResponse,
};
use crate::{
    app_error::{APIError, log_err_convert},
    playground::{model::PlaygroundCheckoutData, store},
};

pub async fn claim_token(
    pool: &PgPool,
    claimed_user_id: Uuid,
    token: Uuid,
    coupons: Option<Vec<Uuid>>,
) -> Result<UserPlaygroundPurchaseResponse, APIError> {
    let mut tx = pool
        .begin()
        .await
        .map_err(|e| log_err_convert("cannot begin transaction", e, APIError::InternalErr))?;

    let claimed = store::update_claimed_status(&mut tx, claimed_user_id, token).await?;
    if let Some(token_info) = claimed {
        let result = if let Some(pg_coupons) = coupons {
            let total_coupons = pg_coupons.len() as i32;
            if total_coupons == 0 {
                tracing::debug!("frontend sent empty json array");
                return Err(APIError::BadRequest(
                    "should not send empty array".to_string(),
                ));
            }
            if pg_coupons.iter().collect::<HashSet<_>>().len() != pg_coupons.len() {
                tracing::debug!("duplicate coupons");
                return Err(APIError::BadRequest("duplicate coupon ids".to_string()));
            }

            let status = store::get_coupon_status(&mut tx, claimed_user_id, &pg_coupons).await?;
            if status.len() != pg_coupons.len() {
                tracing::debug!("coupons not in database");
                return Err(APIError::InvalidCoupons);
            }
            let invalid_coupon_count = status.iter().filter(|x| x.is_expired || x.is_used).count();
            if invalid_coupon_count != 0 {
                tracing::debug!("expired or used coupons");
                return Err(APIError::InvalidCoupons);
            }
            if total_coupons > token_info.total_quantity {
                return Err(APIError::InvalidCouponAmount);
            }
            let purchased_quantity = token_info.total_quantity - total_coupons;
            let free_quantity = total_coupons;
            store::update_playground_account(
                &mut tx,
                claimed_user_id,
                purchased_quantity,
                free_quantity,
            )
            .await?;
            store::update_coupon_status(&mut tx, claimed_user_id, &pg_coupons).await?;
            let purchased: Option<UserPlaygroundPurchase> = if purchased_quantity > 0 {
                Some(
                    store::insert_playground_purchase(
                        &mut tx,
                        claimed_user_id,
                        purchased_quantity,
                        false,
                        token_info.created_by,
                        token_info.unit_price,
                    )
                    .await?
                    .into(),
                )
            } else {
                None
            };

            let free = store::insert_playground_purchase(
                &mut tx,
                claimed_user_id,
                free_quantity,
                true,
                token_info.created_by,
                token_info.unit_price,
            )
            .await?
            .into();
            UserPlaygroundPurchaseResponse {
                purchased_record: purchased,
                free_record: Some(free),
            }
        } else {
            store::update_playground_account(
                &mut tx,
                claimed_user_id,
                token_info.total_quantity,
                0,
            )
            .await?;
            let purchased = store::insert_playground_purchase(
                &mut tx,
                claimed_user_id,
                token_info.total_quantity,
                false,
                token_info.created_by,
                token_info.unit_price,
            )
            .await?
            .into();
            UserPlaygroundPurchaseResponse {
                purchased_record: Some(purchased),
                free_record: None,
            }
        };
        tx.commit()
            .await
            .map_err(|e| log_err_convert("failed to commit", e, APIError::InternalErr))?;
        Ok(result)
    } else {
        let row = store::get_expired_status(&mut tx, token).await?;
        let Some(r) = row else {
            return Err(APIError::NotFound);
        };
        if r.expired || r.status == "expired" {
            tracing::warn!("claiming on expired token: {token}");
            Err(APIError::TokenExpired)
        } else {
            Err(APIError::BadRequest("token already claimed".to_string()))
        }
    }
}

pub async fn claim_free_ticket(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<UserPlaygroundAccount, APIError> {
    let mut tx = pool
        .begin()
        .await
        .map_err(|e| log_err_convert("cannot begin transaction", e, APIError::InternalErr))?;
    let user_playground_account =
        store::get_user_playground_account_for_update(&mut tx, user_id).await?;
    let required_amount = 10;
    if user_playground_account.free_ticket_eligible >= required_amount {
        store::insert_ticket_coupon(&mut tx, user_id).await?;
        let result = store::deduct_eligible_tickets(&mut tx, user_id, required_amount)
            .await?
            .into();
        tx.commit()
            .await
            .map_err(|e| log_err_convert("failed to commit", e, APIError::InternalErr))?;
        Ok(result)
    } else {
        tx.rollback()
            .await
            .map_err(|e| log_err_convert("failed to rollback", e, APIError::InternalErr))?;
        Err(APIError::BadRequest("not enough tickets".to_string()))
    }
}

pub async fn create_playground_account(
    tx: &mut Transaction<'_, Postgres>,
    user_id: Uuid,
) -> Result<(), APIError> {
    store::insert_playground_account(tx, user_id, 0, 0, 0).await?;
    Ok(())
}

pub async fn get_playground_checkout_data_service(
    pool: &PgPool,
    user_id: Option<Uuid>,
    token_id: Uuid,
) -> Result<PlaygroundCheckoutData, APIError> {
    let (token_info, available_coupons) =
        tokio::try_join!(store::get_claim_token_info(pool, token_id), async {
            match user_id {
                Some(uid) => store::get_available_user_coupons(pool, uid).await,
                None => Ok(Vec::new()),
            }
        })?;
    let result = PlaygroundCheckoutData::new(user_id, token_info, available_coupons)
        .ok_or(APIError::InternalErr)?;
    Ok(result)
}

pub async fn get_playground_checkout_data_admin_service(
    pool: &PgPool,
    token_id: Uuid,
) -> Result<PlaygroundCheckoutDataAdmin, APIError> {
    let Some(token_info) = store::get_admin_token_info(pool, token_id).await? else {
        return Err(APIError::NotFound);
    };

    let (total_price, available_coupons) = match token_info.claimed_by {
        Some(user_id) => {
            let coupons = store::get_available_user_coupons(pool, user_id).await?;
            let coupons_redeemed = coupons.len().min(token_info.total_quantity as usize) as i32;
            let charged_quantity = token_info.total_quantity - coupons_redeemed;
            let total_price = Decimal::from_i32(charged_quantity)
                .ok_or(APIError::InternalErr)?
                .mul(token_info.unit_price);
            (Some(total_price), coupons)
        }
        None => (None, Vec::new()),
    };

    Ok(PlaygroundCheckoutDataAdmin {
        claimed_by: token_info.claimed_by,
        total_quantity: token_info.total_quantity,
        unit_price: token_info.unit_price,
        total_price,
        available_coupons,
        expires_at: token_info.expires_at,
        expired: token_info.expired,
    })
}

pub async fn get_admin_purchases_paginated(
    pool: &PgPool,
    filter: &AdminPlaygroundPurchasesQuery,
    limit: i64,
    offset: i64,
) -> Result<(Vec<AdminPlaygroundPurchase>, i64), APIError> {
    let (rows, total_items) =
        store::get_admin_purchases_paginated(pool, filter, limit, offset).await?;
    Ok((rows.into_iter().map(Into::into).collect(), total_items))
}

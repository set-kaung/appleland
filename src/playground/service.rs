use sqlx::{PgPool, Postgres, Transaction};
use std::collections::HashSet;
use uuid::Uuid;

use super::model::{UserPlaygroundAccount, UserPlaygroundPurchase, UserPlaygroundPurchaseResponse};
use crate::{
    app_error::{APIError, log_err_convert},
    playground::store,
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
            store::update_expire_status(&mut tx, token).await?;
            tx.commit()
                .await
                .map_err(|e| log_err_convert("failed to commit", e, APIError::InternalErr))?;
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

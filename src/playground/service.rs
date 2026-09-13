// use sqlx::{PgPool};
// use uuid::Uuid;

// use super::model::UserPlaygroundPurchase;
// use crate::{
//     app_error::{APIError, log_err_convert},
//     playground::store,
// };

// pub async fn claim_token(
//     pool: &PgPool,
//     claimed_user_id: Uuid,
//     token: Uuid,
// ) -> Result<UserPlaygroundPurchase, APIError> {
//     let mut tx = pool
//         .begin()
//         .await
//         .map_err(|e| log_err_convert("cannot begin transaction", e, APIError::InternalErr))?;
//     let claimed = store::update_claimed_status(&mut tx, claimed_user_id, token).await?;
//     if let Some(token_info) = claimed {
//         store::update_user_tokens(&mut tx, token_info).await?;

//         tx.commit()
//             .await
//             .map_err(|e| log_err_convert("failed to commit", e, APIError::InternalErr))?;
//     } else {
//         let row = store::get_expired_status(&mut tx, token).await?;
//         match row {
//             None => Err(APIError::NotFound),
//             Some(r) if r.expired || r.status == "expired" => {
//                 store::update_expire_status(&mut tx, token);
//                 tx.commit()
//                     .await
//                     .map_err(|e| log_err_convert("failed to commit", e, APIError::InternalErr))?;
//                 tracing::warn!("claiming on expired token: {token}");
//                 Err(APIError::TokenExpired)
//             }
//             Some(_) => Err(APIError::BadRequest("token already claimed".to_string())),
//         }
//     }
// }

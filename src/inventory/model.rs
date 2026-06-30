use time::OffsetDateTime;
use uuid::Uuid;
pub struct Inventory {
    pub id: Uuid,
    pub product_id: Uuid,
    pub quantity: i64,
    pub created_by: Uuid,
    pub update_by: Uuid,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

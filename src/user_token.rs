use crate::schema::user_tokens;
use crate::user::User;

#[derive(Identifiable, Associations, Serialize, Deserialize, Queryable)]
#[belongs_to(User)]
pub struct UserToken {
    pub id: i32,
    pub token: String,
    pub user_id: i32,
    pub service: String,
}

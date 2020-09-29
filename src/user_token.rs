use crate::schema::user_tokens;
use crate::user::User;
use diesel::{self, prelude::*};

#[derive(Identifiable, Associations, Serialize, Deserialize, Queryable)]
#[belongs_to(User)]
pub struct UserToken {
    pub id: i32,
    pub token: String,
    pub user_id: i32,
    pub service: String,
}

#[derive(Serialize, Deserialize, Insertable)]
#[table_name = "user_tokens"]
struct InsertableToken {
    token: String,
    user_id: i32,
    service: String,
}

pub fn insert(conn: &PgConnection, user: &User, token_string: String) -> QueryResult<UserToken> {
    use crate::schema::user_tokens::dsl::*;
    let insertable_token = InsertableToken {
        token: token_string,
        user_id: user.id,
        service: "Pipefy".into(),
    };

    diesel::insert_into(user_tokens)
        .values(&insertable_token)
        .get_result(conn)
}

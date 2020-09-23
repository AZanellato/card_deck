use crate::schema::users;
use diesel::{self, prelude::*};

#[derive(Identifiable, Debug, Serialize, Deserialize, Queryable)]
pub struct User {
    pub id: i32,
    pub email: String,
    pub name: String,
    pub hash_password: String,
}

/// This represents a deck being inserted into the database, without the auto-generated fields
#[derive(Serialize, Deserialize, Insertable)]
#[table_name = "users"]
pub struct InsertableUser {
    pub email: String,
    pub name: String,
    pub hash_password: String,
}

pub fn create(conn: &PgConnection, insertable_user: InsertableUser) -> User {
    use crate::schema::users::dsl::*;
    diesel::insert_into(users)
        .values(&insertable_user)
        .get_result(conn)
        .expect("Error saving user")
}

pub fn fetch_by_email(conn: &PgConnection, input_email: &str) -> QueryResult<User> {
    use crate::schema::users::dsl::*;
    users.filter(email.eq(input_email)).first(conn)
}

pub fn fetch_by_id(conn: &PgConnection, input_id: i32) -> QueryResult<User> {
    use crate::schema::users::dsl::*;
    users.find(input_id).first(conn)
}

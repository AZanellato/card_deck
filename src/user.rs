use crate::rocket::request::{FromRequest, Outcome, Request};
use crate::schema::users;
use crate::DeckDbConn;
use diesel::{self, prelude::*};
use rocket::outcome::IntoOutcome;

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

impl<'a, 'r> FromRequest<'a, 'r> for User {
    type Error = ();

    fn from_request(request: &'a Request<'r>) -> Outcome<User, ()> {
        let conn = request.guard::<DeckDbConn>()?;

        request
            .cookies()
            .get_private("user_id")
            .and_then(|cookie| cookie.value().parse().ok())
            .and_then(|id| fetch_by_id(&*conn, id).ok())
            .or_forward(())
    }
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

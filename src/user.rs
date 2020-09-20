use crate::schema::users;
// use diesel::{self, prelude::*};

#[derive(Identifiable, Serialize, Deserialize, Queryable)]
pub struct User {
    pub id: i32,
    pub email: String,
    pub name: i32,
}

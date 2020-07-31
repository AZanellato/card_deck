use crate::schema::decks;
use crate::schema::decks::dsl::*;
use diesel::{self, prelude::*};

#[derive(Identifiable, Serialize, Deserialize, Queryable)]
pub struct Deck {
    pub id: i32,
    pub title: String,
    pub created_by: i32,
}

// card_ids: Vec<usize>,
/// This represents a deck being inserted into the database, without the auto-generated fields
#[derive(Deserialize, Insertable)]
#[table_name = "decks"]
pub struct InsertableDeck {
    pub title: String,
    pub created_by: i32,
}

impl Deck {
    pub fn by_id(conn: &PgConnection, input_id: i32) -> QueryResult<Deck> {
        decks.find(input_id).get_result(conn)
    }
}

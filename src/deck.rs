use crate::schema::decks;
use diesel::{self, insert_into, prelude::*};

#[derive(Identifiable, Serialize, Deserialize, Queryable)]
pub struct Deck {
    pub id: i32,
    pub title: String,
    pub created_by: i32,
}

// card_ids: Vec<usize>,
/// This represents a deck being inserted into the database, without the auto-generated fields
#[derive(Serialize, Deserialize, Insertable)]
#[table_name = "decks"]
pub struct InsertableDeck {
    pub title: String,
    pub created_by: i32,
}

pub fn by_id(conn: &PgConnection, input_id: i32) -> QueryResult<Deck> {
    use crate::schema::decks::dsl::*;
    decks.find(input_id).get_result(conn)
}

pub fn all(conn: &PgConnection) -> Vec<Deck> {
    use crate::schema::decks::dsl::*;
    let empty_vec: Vec<Deck> = vec![];
    decks.load::<Deck>(conn).unwrap_or(empty_vec)
}
pub fn create(conn: &PgConnection, insertable: InsertableDeck) -> QueryResult<Deck> {
    use crate::schema::decks::dsl::*;

    insert_into(decks).values(&insertable).get_result(conn)
}

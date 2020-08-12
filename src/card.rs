use super::deck::Deck;
use super::schema::cards;
use chrono::NaiveDateTime;
use diesel::{self, insert_into, prelude::*};

#[derive(Associations, Identifiable, Serialize, Deserialize, Queryable)]
#[belongs_to(Deck)]
pub struct Card {
    pub id: i32,
    pub title: String,
    pub deck_id: i32,
    pub finished_at: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

/// This represents a card being inserted into the database, without the auto-generated fields
#[derive(Serialize, Deserialize, Insertable)]
#[table_name = "cards"]
pub struct InsertableCard {
    pub title: String,
    pub deck_id: i32,
    pub finished_at: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Serialize, Deserialize, AsChangeset)]
#[table_name = "cards"]
pub struct UpdateableCard {
    pub title: Option<String>,
    pub deck_id: Option<i32>,
    pub finished_at: Option<Option<NaiveDateTime>>,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
}

pub fn by_id(conn: &PgConnection, input_id: i32) -> QueryResult<Card> {
    use crate::schema::cards::dsl::*;
    cards.find(input_id).get_result(conn)
}

pub fn by_deck(conn: &PgConnection, deck_id: i32) -> Vec<Card> {
    use crate::schema::cards::dsl::*;
    let empty_vec: Vec<Card> = vec![];
    cards
        .filter(deck_id.eq(deck_id))
        .load::<Card>(conn)
        .unwrap_or(empty_vec)
}

pub fn create(conn: &PgConnection, insertable: InsertableCard) -> QueryResult<Card> {
    use crate::schema::cards::dsl::*;

    insert_into(cards).values(&insertable).get_result(conn)
}

pub fn update(conn: &PgConnection, updateable: UpdateableCard) -> QueryResult<usize> {
    use crate::schema::cards::dsl::*;

    diesel::update(cards).set(&updateable).execute(conn)
}

use super::deck::Deck;
use super::schema::cards;
use super::user_token::UserToken;
use anyhow::Result;
use chrono::NaiveDateTime;
use chrono::{DateTime, Utc};
use diesel::{self, insert_into, prelude::*};
mod pipefy;

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
    cards
        .filter(deck_id.eq(deck_id))
        .load::<Card>(conn)
        .unwrap_or_else(|_| Vec::new())
}

pub fn create(conn: &PgConnection, insertable: InsertableCard) -> QueryResult<Card> {
    use crate::schema::cards::dsl::*;

    insert_into(cards).values(&insertable).get_result(conn)
}

pub fn update(
    conn: &PgConnection,
    user_token: UserToken,
    updateable: UpdateableCard,
) -> QueryResult<usize> {
    use crate::schema::cards::dsl::*;

    diesel::update(cards).set(&updateable).execute(conn)
}

pub fn from_pipefy_to_deck(
    conn: &PgConnection,
    user_token: UserToken,
    card_id: i32,
    deck: &Deck,
) -> Result<usize> {
    let api_token = user_token.token;
    let pipefy_card = pipefy::by_id(&api_token, card_id)?;
    let card = InsertableCard {
        title: pipefy_card.title,
        deck_id: deck.id,
        finished_at: pipefy_card.finished_at.map(|dt| dt.naive_utc()),
        created_at: pipefy_card.created_at.naive_utc(),
        updated_at: pipefy_card.updated_at.naive_utc(),
    };
    let insertion_result = diesel::insert_into(cards::table)
        .values(&card)
        .execute(conn);

    insertion_result.map_err(|err| anyhow::Error::new(err))
}

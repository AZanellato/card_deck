use super::deck::Deck;
use super::schema::cards;
use super::user_token::UserToken;
use anyhow::Result;
use chrono::NaiveDateTime;
use diesel::{self, insert_into, prelude::*};
mod pipefy;

#[derive(Associations, Identifiable, Serialize, Debug, Deserialize, Queryable)]
#[belongs_to(Deck)]
pub struct Card {
    pub id: i32,
    pub title: String,
    pub deck_id: i32,
    pub finished_at: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub started_at: Option<NaiveDateTime>,
}

/// This represents a card being inserted into the database, without the auto-generated fields
#[derive(Serialize, Deserialize, Insertable, Debug)]
#[table_name = "cards"]
pub struct InsertableCard {
    pub title: String,
    pub deck_id: i32,
    pub finished_at: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub started_at: Option<NaiveDateTime>,
}

#[derive(Serialize, Deserialize, AsChangeset)]
#[table_name = "cards"]
pub struct UpdateableCard {
    pub title: Option<String>,
    pub deck_id: Option<i32>,
    pub finished_at: Option<Option<NaiveDateTime>>,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
    pub started_at: Option<Option<NaiveDateTime>>,
}

pub struct Days(i64);

pub struct Metrics {
    lead_time: Option<Days>,
    cycle_time: Option<Days>,
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

pub fn count_by_deck(conn: &PgConnection, i_deck_id: i32) -> i64 {
    use crate::schema::cards::dsl::*;

    let count = cards
        .filter(deck_id.eq(i_deck_id))
        .count()
        .get_result(conn)
        .unwrap_or(0);

    count
}

pub fn throughput_by_deck(conn: &PgConnection, i_deck_id: i32, n_weeks: u32) -> usize {
    use crate::schema::cards::dsl::*;

    let today = chrono::offset::Local::today().naive_utc();
    let date = today - chrono::Duration::weeks(n_weeks.into());
    let t = chrono::NaiveTime::from_hms_milli(00, 00, 00, 000);
    let date_time = date.and_time(t);
    let throughput = cards
        .filter(deck_id.eq(i_deck_id))
        .filter(finished_at.gt(date_time))
        .count()
        .get_result(conn)
        .unwrap_or(0);

    throughput as usize
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
) -> Result<Card> {
    let api_token = user_token.token;
    let pipefy_card = pipefy::by_id(&api_token, card_id)?;
    let starting_phase_id = deck.starting_phase_id;
    let start_phase_history = pipefy_card.phases_history.iter().find(|phase_history| {
        match phase_history.phase.id.parse::<i32>() {
            Ok(id) => id == starting_phase_id,
            Err(_) => false,
        }
    });

    let started_at = start_phase_history.map(|history| history.first_time_in.naive_utc());

    dbg!(&started_at);

    let card = InsertableCard {
        title: pipefy_card.title,
        deck_id: deck.id,
        finished_at: pipefy_card.finished_at.map(|dt| dt.naive_utc()),
        created_at: pipefy_card.created_at.naive_utc(),
        updated_at: pipefy_card.updated_at.naive_utc(),
        started_at,
    };

    diesel::insert_into(cards::table)
        .values(&card)
        .get_result::<Card>(conn)
        .map_err(|err| anyhow::Error::new(err))
}

impl Card {
    pub fn metrics(&self) -> Metrics {
        let lead_time = self.lead_time();
        let cycle_time = self.cycle_time();

        Metrics {
            lead_time,
            cycle_time,
        }
    }

    fn cycle_time(&self) -> Option<Days> {
        let finished_at = self.finished_at?;
        let duration = self.created_at - finished_at;
        Some(Days(duration.num_days()))
    }

    pub fn lead_time(&self) -> Option<Days> {
        let finished_at = self.finished_at?;
        let started_at = self.started_at?;
        let duration = started_at - finished_at;
        Some(Days(duration.num_days()))
    }
}

use crate::schema::decks;
use diesel::{self, insert_into, prelude::*};

#[derive(Identifiable, Serialize, Deserialize, Queryable)]
pub struct Deck {
    pub id: i32,
    pub title: String,
    pub created_by: i32,
    pub pipe_id: i32,
    pub starting_phase_id: i32,
}

/// This represents a deck being inserted into the database, without the auto-generated fields
#[derive(Serialize, Deserialize, Insertable)]
#[table_name = "decks"]
pub struct InsertableDeck {
    pub title: String,
    pub created_by: i32,
    pub pipe_id: i32,
    pub starting_phase_id: i32,
}

#[derive(Serialize, Deserialize, Insertable)]
#[table_name = "decks"]
pub struct UpdateableDeck {
    pub title: String,
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

impl Deck {
    pub fn throughput(&self, conn: &PgConnection) -> usize {
        let cards = crate::card::by_deck(conn, self.id);
        13
    }

    pub fn lead_time(&self, conn: &PgConnection) -> usize {
        let cards = crate::card::by_deck(conn, self.id);
        let finished_cards_size = cards
            .iter()
            .filter(|card| card.finished_at.is_some())
            .count();

        if finished_cards_size == 0 {
            return 0;
        }

        let cards_lead_times: i64 = cards
            .into_iter()
            .map(|card| {
                card.finished_at
                    .map(|f_date| f_date.date())
                    .unwrap_or(chrono::offset::Local::today().naive_utc())
                    - card.created_at.date()
            })
            .map(|duration| duration.num_days())
            .sum();

        cards_lead_times as usize / finished_cards_size
    }
}

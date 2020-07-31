use crate::deck::{Deck, InsertableDeck};
use crate::DeckDbConn;
use diesel::{self, prelude::*};
use rocket_contrib::json::Json;
use rocket_contrib::templates::Template;

#[derive(Serialize, Debug)]
struct DeckTemplate {
    title: Option<String>,
    id: Option<i32>,
    error_message: Option<String>,
}

#[get("/")]
pub fn index() -> &'static str {
    "Hello, world!"
}

#[get("/decks/<id>/json")]
pub fn get_decks_as_json(conn: crate::DeckDbConn, id: i32) -> Result<Json<Deck>, String> {
    Deck::by_id(&*conn, id)
        .map_err(|err| match err {
            diesel::result::Error::NotFound => (format!("No deck with id: {}", id)).into(),
            _ => {
                println!("Error querying page views: {:?}", err);
                "Error querying page views from the database".into()
            }
        })
        .map(Json)
}

#[get("/decks/<id>")]
pub fn get_decks(conn: crate::DeckDbConn, id: i32) -> Template {
    let possible_deck: Result<Deck, String> = Deck::by_id(&*conn, id).map_err(|err| match err {
        diesel::result::Error::NotFound => format!("No deck with id: {}", id),
        _ => "Error on the database".into(),
    });
    let context = match possible_deck {
        Ok(deck) => DeckTemplate {
            title: Some(deck.title),
            id: Some(deck.id),
            error_message: None,
        },
        Err(error_message) => DeckTemplate {
            title: None,
            id: None,
            error_message: Some(error_message),
        },
    };
    dbg!(&context);
    Template::render("deck", &context)
}

// #[post("/decks")]
// pub fn post_deck(conn: DeckDbConn, data: InsertableDeck) -> Template {
//     let context = match possible_deck {
//         Ok(deck) => DeckTemplate {
//             title: Some(deck.title),
//             id: Some(deck.id),
//             error_message: None,
//         },
//         Err(error_message) => DeckTemplate {
//             title: None,
//             id: None,
//             error_message: Some(error_message),
//         },
//     };
//     dbg!(&context);
//     Template::render("deck", &context)
// }

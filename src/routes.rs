use crate::card;
use crate::deck::{self, Deck, InsertableDeck};
use crate::user::User;
use crate::user_token::UserToken;
use crate::DeckDbConn;
use diesel::{self, prelude::*};
use rocket::request::Form;
use rocket_contrib::json::Json;
use rocket_contrib::templates::Template;

#[derive(Serialize, Debug)]
struct DeckTemplate {
    title: Option<String>,
    id: Option<i32>,
    error_message: Option<String>,
    // This key tells handlebars which template is the parent.
    parent: &'static str,
}

#[derive(Debug, FromForm)]
pub struct FormDeck {
    title: String,
}

#[derive(Debug, FromForm)]
pub struct FormCardId {
    card_id: i32,
}

#[get("/")]
pub fn index() -> &'static str {
    "Hello, world!"
}

#[get("/decks")]
pub fn get_decks(conn: crate::DeckDbConn) -> Json<Vec<Deck>> {
    Json(deck::all(&*conn))
}

#[get("/decks/<id>/json", format = "json")]
pub fn get_deck_as_json(conn: crate::DeckDbConn, id: i32) -> Result<Json<Deck>, String> {
    deck::by_id(&*conn, id)
        .map_err(|err| match err {
            diesel::result::Error::NotFound => (format!("No deck with id: {}", id)).into(),
            _ => {
                println!("Error querying page views: {:?}", err);
                "Error querying page views from the database".into()
            }
        })
        .map(Json)
}

#[get("/decks/<id>", format = "html")]
pub fn get_deck(conn: DeckDbConn, id: i32) -> Template {
    let possible_deck: Result<Deck, String> = deck::by_id(&*conn, id).map_err(|err| match err {
        diesel::result::Error::NotFound => format!("No deck with id: {}", id),
        _ => "Error on the database".into(),
    });
    let context = match possible_deck {
        Ok(deck) => DeckTemplate {
            title: Some(deck.title),
            id: Some(deck.id),
            error_message: None,
            parent: "layout",
        },
        Err(error_message) => DeckTemplate {
            title: None,
            id: None,
            error_message: Some(error_message),
            parent: "layout",
        },
    };
    Template::render("deck", &context)
}
#[post("/decks/<id>", data = "<card_form>")]
pub fn add_card_to_deck(
    conn: DeckDbConn,
    user: User,
    id: i32,
    card_form: Form<FormCardId>,
) -> Template {
    let possible_deck: Result<Deck, String> = deck::by_id(&*conn, id).map_err(|err| match err {
        diesel::result::Error::NotFound => format!("No deck with id: {}", id),
        _ => "Error on the database".into(),
    });

    let deck = match possible_deck {
        Ok(deck) => deck,
        Err(error_message) => {
            return Template::render(
                "deck",
                &DeckTemplate {
                    title: None,
                    id: None,
                    error_message: Some(error_message),
                    parent: "layout",
                },
            )
        }
    };
    let query_for_user_token = UserToken::belonging_to(&user).first(&*conn);

    if let Err(_) = query_for_user_token {
        return Template::render(
            "deck",
            DeckTemplate {
                title: None,
                id: None,
                error_message: Some("No Token Found".into()),
                parent: "layout",
            },
        );
    }

    let user_token = query_for_user_token.unwrap();
    card::from_pipefy_to_deck(&conn, user_token, card_form.card_id, deck);

    Template::render(
        "deck",
        DeckTemplate {
            title: Some(deck.title),
            id: Some(deck.id),
            error_message: None,
            parent: "layout",
        },
    )
}

#[post("/decks", data = "<form_deck>")]
pub fn post_deck(conn: DeckDbConn, form_deck: Form<FormDeck>) -> Template {
    let insertable_deck = InsertableDeck {
        title: form_deck.into_inner().title,
        created_by: 1,
    };
    let result = deck::create(&*conn, insertable_deck);
    let context = DeckTemplate {
        title: None,
        id: None,
        error_message: Some("Not saved".into()),
        parent: "layout",
    };
    Template::render("deck", &context)
}

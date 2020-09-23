use crate::card;
use crate::deck::{self, Deck, InsertableDeck};
use crate::user::{self, InsertableUser, User};
use crate::user_token::UserToken;
use crate::DeckDbConn;
use diesel::{self, prelude::*};
use djangohashers::{check_password, make_password};
use rocket::http::{Cookie, Cookies};
use rocket::outcome::IntoOutcome;
use rocket::request::{Form, FromRequest, Outcome, Request};
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

#[derive(FromForm, Debug)]
pub struct UserLogin {
    pub email: String,
    pub password: String,
}

#[derive(FromForm, Debug)]
pub struct UserCreate {
    pub email: String,
    pub password: String,
    pub name: String,
}

#[derive(Debug, FromForm)]
pub struct FormCardId {
    card_id: i32,
}

impl<'a, 'r> FromRequest<'a, 'r> for User {
    type Error = ();

    fn from_request(request: &'a Request<'r>) -> Outcome<User, ()> {
        let conn = request.guard::<DeckDbConn>()?;

        request
            .cookies()
            .get_private("user_id")
            .and_then(|cookie| cookie.value().parse().ok())
            .and_then(|id| user::fetch_by_id(&*conn, id).ok())
            .or_forward(())
    }
}
// routes
#[get("/")]
pub fn index() -> &'static str {
    "Hello, world!"
}

#[get("/decks")]
pub fn get_decks(conn: DeckDbConn, user: User) -> Json<Vec<Deck>> {
    use crate::schema::decks::dsl::*;

    let results = decks
        .filter(created_by.eq(user.id))
        .load::<Deck>(&*conn)
        .unwrap_or(vec![]);

    Json(results)
}

#[get("/decks/<id>/json", format = "json")]
pub fn get_deck_as_json(conn: DeckDbConn, id: i32) -> Result<Json<Deck>, String> {
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
#[get("/me")]
pub fn user_info(user: User) -> Json<(String, String)> {
    Json((user.name, user.email))
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
    let inserted_result = card::from_pipefy_to_deck(&conn, user_token, card_form.card_id, &deck);

    match inserted_result {
        Ok(_) => Template::render(
            "deck",
            DeckTemplate {
                title: Some(deck.title),
                id: Some(deck.id),
                error_message: None,
                parent: "layout",
            },
        ),
        Err(_) => Template::render(
            "deck",
            DeckTemplate {
                title: None,
                id: None,
                error_message: Some("Error when inserting the card".into()),
                parent: "layout",
            },
        ),
    }
}

#[post("/decks", data = "<form_deck>")]
pub fn post_deck(conn: DeckDbConn, user: User, form_deck: Form<FormDeck>) -> Template {
    let insertable_deck = InsertableDeck {
        title: form_deck.into_inner().title,
        created_by: user.id,
    };
    let result = deck::create(&*conn, insertable_deck);
    match result {
        Ok(deck) => {
            let context = DeckTemplate {
                title: Some(deck.title),
                id: Some(deck.id),
                error_message: None,
                parent: "layout",
            };
            Template::render("deck", &context)
        }
        Err(err) => {
            let context = DeckTemplate {
                title: None,
                id: None,
                error_message: Some(err.to_string()),
                parent: "layout",
            };
            Template::render("deck", &context)
        }
    }
}

#[post("/users/login", data = "<login_info>")]
pub fn login_user(
    conn: DeckDbConn,
    login_info: Form<UserLogin>,
    mut cookies: Cookies,
) -> Json<Option<i32>> {
    let maybe_user = user::fetch_by_email(&conn, &login_info.email);
    if let Err(_) = maybe_user {
        return Json(None);
    }

    let user = maybe_user.unwrap();

    match check_password(&login_info.password, &user.hash_password) {
        Ok(valid) => {
            if valid {
                cookies.add_private(Cookie::new("user_id", user.id.to_string()));
                Json(Some(user.id))
            } else {
                Json(None)
            }
        }
        Err(_) => Json(None),
    }
}

#[post("/users/create", data = "<creation_info>")]
pub fn create_user(
    conn: DeckDbConn,
    creation_info: Form<UserCreate>,
) -> Json<Result<User, String>> {
    let maybe_user = user::fetch_by_email(&conn, &creation_info.email);
    if let Ok(_) = maybe_user {
        return Json(Err("User already exists".into()));
    }

    let hash_password = make_password(&creation_info.password);

    let user_info = Form::into_inner(creation_info);
    let insertable_user = InsertableUser {
        email: user_info.email,
        name: user_info.name,
        hash_password,
    };
    Json(Ok(user::create(&*conn, insertable_user)))
}

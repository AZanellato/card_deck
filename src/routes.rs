use crate::card::{self, Card};
use crate::deck::{self, Deck, InsertableDeck};
use crate::user::{self, InsertableUser, User};
use crate::user_token::{self, UserToken};
use crate::DeckDbConn;
use diesel::{self, prelude::*};
use djangohashers::{check_password, make_password};
use rocket::http::{Cookie, Cookies};
use rocket::request::Form;
use rocket_contrib::json::Json;
use rocket_contrib::templates::Template;

#[derive(Serialize, Debug)]
struct DeckTemplate {
    title: Option<String>,
    id: Option<i32>,
    error_message: Option<String>,
    cards: Vec<Card>,
    cards_count: i64,
    lead_time: usize,
    throughput: usize,
    // This key tells handlebars which template is the parent.
    parent: &'static str,
}

#[derive(Debug, FromForm)]
pub struct FormDeck {
    title: String,
    pipe_id: i32,
}

#[derive(Debug, FromForm)]
pub struct Token {
    token: String,
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
pub fn get_deck(conn: DeckDbConn, id: i32, _u: User) -> Template {
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
                    cards: vec![],
                    cards_count: 0,
                    lead_time: 7,
                    throughput: 17,
                    parent: "layout",
                },
            )
        }
    };
    let cards: Vec<Card> = Card::belonging_to(&deck)
        .get_results(&*conn)
        .unwrap_or_else(|_| vec![]);

    let cards_count = card::count_by_deck(&conn, deck.id);
    let lead_time = deck.lead_time(&conn);

    let context = DeckTemplate {
        title: Some(deck.title),
        id: Some(deck.id),
        cards_count,
        lead_time,
        throughput: 17,
        cards,
        error_message: None,
        parent: "layout",
    };

    Template::render("deck", &context)
}
#[get("/me")]
pub fn user_info(user: User) -> Json<(String, String)> {
    Json((user.name, user.email))
}

#[get("/login")]
pub fn login_page() -> Template {
    Template::render(
        "login",
        &DeckTemplate {
            title: None,
            id: None,
            error_message: None,
            parent: "layout",
            cards: vec![],
            cards_count: 0,
            lead_time: 0,
            throughput: 17,
        },
    )
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
                    cards: vec![],
                    cards_count: 0,
                    lead_time: 0,
                    throughput: 17,
                },
            )
        }
    };
    let mut cards: Vec<Card> = Card::belonging_to(&deck)
        .get_results(&*conn)
        .unwrap_or_else(|_| vec![]);

    let query_for_user_token = UserToken::belonging_to(&user).first(&*conn);
    let lead_time = deck.lead_time(&conn);

    let cards_count = card::count_by_deck(&conn, deck.id);
    if let Err(_) = query_for_user_token {
        let context = DeckTemplate {
            title: Some(deck.title),
            id: Some(deck.id),
            cards_count,
            lead_time,
            throughput: 17,
            cards,
            error_message: None,
            parent: "layout",
        };
        return Template::render("deck", context);
    }

    let user_token = query_for_user_token.unwrap();
    let inserted_result = card::from_pipefy_to_deck(&conn, user_token, card_form.card_id, &deck);

    let cards_count = card::count_by_deck(&conn, deck.id);
    let lead_time = deck.lead_time(&conn);
    match inserted_result {
        Ok(card) => {
            cards.push(card);
            let cards_count = card::count_by_deck(&conn, deck.id);
            Template::render(
                "deck",
                DeckTemplate {
                    title: Some(deck.title),
                    id: Some(deck.id),
                    cards_count,
                    lead_time,
                    throughput: 17,
                    cards,
                    error_message: None,
                    parent: "layout",
                },
            )
        }
        Err(_) => Template::render(
            "deck",
            DeckTemplate {
                title: None,
                id: None,
                cards_count,
                lead_time,
                throughput: 17,
                cards,
                error_message: Some("Error when inserting the card".into()),
                parent: "layout",
            },
        ),
    }
}

#[post("/decks", data = "<form_deck>")]
pub fn post_deck(conn: DeckDbConn, user: User, form_deck: Form<FormDeck>) -> Template {
    let ins_deck = |form: FormDeck| InsertableDeck {
        title: form.title,
        created_by: user.id,
        pipe_id: form.pipe_id,
    };
    let result = deck::create(&*conn, ins_deck(form_deck.into_inner()));
    let cards = vec![];
    let cards_count = 0;
    let lead_time = 0;
    let throughput = 0;
    match result {
        Ok(deck) => {
            let cards_count = card::count_by_deck(&conn, deck.id);
            let context = DeckTemplate {
                title: Some(deck.title),
                id: Some(deck.id),
                cards_count,
                lead_time,
                throughput,
                cards,
                error_message: None,
                parent: "layout",
            };
            Template::render("deck", &context)
        }
        Err(err) => {
            let context = DeckTemplate {
                title: None,
                id: None,
                cards_count,
                lead_time,
                throughput,
                cards,
                error_message: Some(err.to_string()),
                parent: "layout",
            };
            Template::render("deck", &context)
        }
    }
}

#[post("/login", data = "<login_info>")]
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

#[post("/users/token", data = "<token_form>")]
pub fn add_token(
    conn: DeckDbConn,
    user: User,
    token_form: Form<Token>,
) -> Json<Result<User, String>> {
    let result = user_token::insert(&*conn, &user, token_form.into_inner().token);
    match result {
        Ok(_) => Json(Ok(user)),
        Err(_) => Json(Err("Something went wrong".into())),
    }
}

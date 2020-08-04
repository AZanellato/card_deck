#![feature(proc_macro_hygiene, decl_macro)]
#[macro_use]
extern crate rocket;
#[macro_use]
extern crate rocket_contrib;
#[macro_use]
extern crate diesel;
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;

use rocket_contrib::templates::Template;

pub mod deck;
pub mod routes;
pub mod schema;

#[database("postgres_db")]
pub struct DeckDbConn(diesel::PgConnection);

fn main() {
    rocket::ignite()
        .attach(DeckDbConn::fairing())
        .attach(Template::fairing())
        .mount(
            "/",
            routes![
                routes::index,
                routes::post_deck,
                routes::get_deck,
                routes::get_deck_as_json,
                routes::get_decks,
                routes::add_card_to_deck
            ],
        )
        .launch();
}

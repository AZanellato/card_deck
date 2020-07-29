#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;
#[macro_use]
extern crate rocket_contrib;

use rocket_contrib::databases::diesel;

#[database("postgres_db")]
struct LogsDbConn(diesel::PgConnection);

fn main() {
    rocket::ignite()
        .attach(LogsDbConn::fairing())
        .mount("/", routes![index])
        .launch();
}
#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

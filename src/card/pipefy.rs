use anyhow::Result;
use chrono::{DateTime, Utc};
use serde_json::{from_str, Value};
use std::collections::HashMap;
use std::error::Error;
use std::fmt::{self, Formatter};

#[derive(Debug, Clone)]
struct Unauthorized;
impl Unauthorized {
    fn new() -> Unauthorized {
        Unauthorized {}
    }
}

impl Error for Unauthorized {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}

impl fmt::Display for Unauthorized {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "Unauthorized access")
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PipefyCard {
    pub title: String,
    #[serde(rename(deserialize = "createdAt"))]
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub finished_at: Option<DateTime<Utc>>,
    pub phases_history: Vec<PhaseHistory>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PhaseHistory {
    #[serde(rename(deserialize = "firstTimeIn"))]
    first_time_in: DateTime<Utc>,
    phase: Phase,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Phase {
    name: String,
    id: String,
}

pub fn by_id(api_key: &str, card_id: i32) -> Result<PipefyCard> {
    let mut query: HashMap<&str, String> = HashMap::new();
    let format_card_query_string = format!(
        "query {{
        card(id: {id}) {{
            title
            createdAt
            updated_at
            finished_at
            phases_history {{
                firstTimeIn
                phase {{
                    name
                    id
                }}
            }}
        }} }}",
        id = card_id
    );
    let card_query_string = String::from(format_card_query_string);
    query.insert("query", card_query_string);
    let text_response = perform_query(api_key, query)?;
    let response_body: Value = serde_json::from_str(&text_response)?;

    let card = serde_json::from_value::<PipefyCard>(response_body["data"]["card"].to_owned());
    match card {
        Ok(card) => Ok(card),
        _ => Err(anyhow::Error::new(Unauthorized::new())),
    }
}

fn perform_query(api_key: &str, query: HashMap<&str, String>) -> Result<String, reqwest::Error> {
    let client = reqwest::blocking::Client::new();
    let res = client
        .post("https://app.pipefy.com/queries")
        .json(&query)
        .bearer_auth(api_key)
        .send()?;

    res.text()
}

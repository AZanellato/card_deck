#[derive(Debug, Serialize, Deserialize)]
pub struct PipefyCard {
    pub title: String,
    pub done: NaiveDateTime,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub phases_history: Vec<PhaseHistory>,
}

pub struct PhaseHistory {
    firstTimeIn: NaiveDateTime,
    phase: Phase,
}

pub struct Phase {
    name: String,
    id: usize,
}

pub fn card_query_and_print(api_key: &str, card_id: i32) -> Result<(), Box<Error>> {
    let mut query: HashMap<&str, String> = HashMap::new();
    let format_card_query_string = format!(
        "query {{
        card(id: {id}) {{
            title
            createdAt
            finished_at
            phases_history {
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
        Ok(card) => {
            dbg!(&card);
            Ok(())
        }
        _ => Err(Box::new(Unauthorized::new())),
    }
}

fn perform_query(api_key: &str, query: HashMap<&str, String>) -> Result<String, reqwest::Error> {
    let client = reqwest::Client::new();
    let mut res = client
        .post("https://app.pipefy.com/queries")
        .json(&query)
        .bearer_auth(api_key)
        .send()?;

    res.text()
}

mod agent;
mod youtube;

use async_openai::Client;
use async_openai::config::OpenAIConfig;
use rocket::{get, routes, State};
use shuttle_runtime::__internals::Context;
use shuttle_runtime::SecretStore;

pub(crate) enum Role {
    AGENT,
    USER,
    SYSTEM,
}

pub(crate) struct Message {
    pub(crate) content: String,
    pub(crate) role: Role,
}

pub(crate) struct Agent {
    pub(crate) system: String,
    pub(crate) model: String,
    pub(crate) history: Vec<Message>,
    pub(crate) client: Client<OpenAIConfig>,
}

struct SecretState {
    openai_api_key: String,
}

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[get("/youtube/<video_id>")]
async fn get_youtube_summary(state: &State<SecretState>, video_id: &str,) -> String {
    youtube::summarize_video(video_id, state.openai_api_key.as_str()).await
}

#[shuttle_runtime::main]
async fn main(#[shuttle_runtime::Secrets] _secrets: SecretStore) -> shuttle_rocket::ShuttleRocket {
    let openai_api_key = _secrets.get("OPENAI_API_KEY").context("OPENAI_API_KEY was not found")?;
    let secret_state = SecretState {
        openai_api_key,
    };

    let rocket = rocket::build().mount("/", routes![index, get_youtube_summary]).manage(secret_state);

    Ok(rocket.into())
}
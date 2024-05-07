mod agent;
mod youtube;

use std::convert::Infallible;
use async_openai::Client;
use async_openai::config::OpenAIConfig;
use rocket::{catch, catchers, get, Request, request, routes, State};
use rocket::http::Status;
use rocket::request::{FromRequest, Outcome};
use rocket::serde::Serialize;
use shuttle_runtime::__internals::Context;
use shuttle_runtime::__internals::tracing_subscriber::fmt::format::Json;
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
    // openai_api_key: String,
    api_key: String,
}

#[derive(Debug)]
struct ApiKey<'r>(&'r str);

#[derive(Debug)]
struct OpenAIKey<'r>(&'r str);

impl<'r> AsRef<str> for OpenAIKey<'r> {
    fn as_ref(&self) -> &str {
        self.0
    }
}

#[derive(Debug)]
enum ApiKeyError {
    Missing,
    Invalid,
}

#[derive(Serialize)]
struct ErrorResponse {
    code: String,
    message: String,
    success: bool,
}

#[catch(400)]
fn bad_request() -> String {
    "Sorry, the request is invalid.".to_string()
}

#[catch(401)]
fn unauthorized() -> String {
    "Sorry, you are not authorized to access this resource.".to_string()
}

#[catch(404)]
fn not_found(req: &Request) -> String {
    format!("Sorry, '{}' is not a valid path.", req.uri())
}

#[catch(500)]
fn internal_error() -> String {
    "Sorry, There is an internal server error".to_string()
}

#[get("/")]
fn index(_key: ApiKey<'_>, _openai_key: OpenAIKey<'_>) -> &'static str {
    "Hello, world!"
}

#[get("/youtube/<video_id>")]
async fn get_youtube_summary<'r>(state: &State<SecretState>, video_id: &str, _key: ApiKey<'_>, _openai_key: OpenAIKey<'_>) -> String {
    youtube::summarize_video(video_id, _openai_key.as_ref()).await
}

#[shuttle_runtime::main]
async fn main(#[shuttle_runtime::Secrets] _secrets: SecretStore) -> shuttle_rocket::ShuttleRocket {
    let api_key = _secrets.get("API_TOKEN").context("API_TOKEN was not found").unwrap();
    let secret_state = SecretState {
        api_key,
    };

    let rocket = rocket::build()
        .register("/", catchers![not_found, internal_error])
        .mount("/", routes![index, get_youtube_summary]).manage(secret_state);

    Ok(rocket.into())
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for ApiKey<'r> {
    type Error = ApiKeyError;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let secret_state = req.rocket().state::<SecretState>().unwrap();

        fn is_valid(key: &str, api_key: &str) -> bool {
            key == api_key
        }

        match req.headers().get_one("Authorization") {
            None => Outcome::Error((Status::Unauthorized, ApiKeyError::Missing)),
            Some(key) if is_valid(key, &secret_state.api_key) => Outcome::Success(ApiKey(key)),
            Some(_) => Outcome::Error((Status::Unauthorized, ApiKeyError::Invalid)),
        }
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for OpenAIKey<'r> {
    type Error = ApiKeyError;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {

        match req.headers().get_one("x-openai-key") {
            None => Outcome::Error((Status::BadRequest, ApiKeyError::Missing)),
            Some(key) => Outcome::Success(OpenAIKey(key)),
        }
    }
}

use async_openai::Client;
use async_openai::config::OpenAIConfig;
use rocket::serde::Deserialize;
use youtube_captions::DigestScraper;
use youtube_captions::format::Format;
use youtube_captions::language_tags::LanguageTag;
use crate::Agent;
use crate::agent::{SUMMARY_PROMPT, SUMMARY_TO_JSON_PROMPT};

#[derive(Deserialize)]
struct Transcript {
    events: Vec<Event>,
}
#[derive(Deserialize)]
struct Event {
    segs: Option<Vec<Segment>>,
}

#[derive(Deserialize)]
struct Segment {
    utf8: String,
}


async fn get_transcript(video: &str) -> String {
    let digest = DigestScraper::new(reqwest::Client::new());
    // Fetch the video
    let scraped = digest.fetch(video, "zh").await.unwrap();

    // Find our preferred language - in this case, english
    let language = LanguageTag::parse("zh").unwrap();

    let captions = scraped.captions.into_iter()
        .find(|caption| language.matches(&caption.lang_tag))
        .unwrap();
    let transcript_json = captions.fetch(Format::JSON3).await.unwrap();

    let root: Transcript = serde_json::from_str(transcript_json.as_str()).unwrap();

    // Collect all utf8 fields from all events and all segments
    let transcript: String = root.events.iter()
        .filter_map(|event| event.segs.as_ref())
        .flatten()
        .map(|segment| segment.utf8.clone()) // Extract the utf8 field of each segment
        .collect::<Vec<String>>()
        .join(" ");

    transcript
}

pub(crate) async fn summarize_video(video: &str, openai_api_key: &str) -> String {
    let client = Client::with_config(
        OpenAIConfig::default(),
    );

    //First, we fetch the transcript for the video
    let transcript = get_transcript(video).await;

    // Then we create our summary agent and have it summarize the video for us
    let mut summarize_agent = Agent {
        system: SUMMARY_PROMPT.to_string(),
        model: "gpt-4-turbo".to_string(),
        history: vec![],
        client: client.clone(),
    };

    let mut summary_to_json_agent = Agent {
        system: SUMMARY_TO_JSON_PROMPT.to_string(),
        model: "gpt-4-turbo".to_string(),
        history: vec![],
        client: client.clone(),
    };

    let summary = summarize_agent.prompt(transcript, openai_api_key).await.unwrap();

    let json = summary_to_json_agent.prompt(summary, openai_api_key).await.unwrap();
    let result = Agent::extract_codeblock(&json);

    result
}



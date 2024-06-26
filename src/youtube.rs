use std::time::Duration;
use async_openai::Client;
use async_openai::config::OpenAIConfig;
use rocket::serde::Deserialize;
use youtube_captions::{CaptionScraper, Digest, DigestScraper};
use youtube_captions::format::Format;
use youtube_captions::language_tags::LanguageTag;
use crate::Agent;
use crate::agent::{get_summary_prompt, get_summary_to_json_prompt};

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

const LANGUAGES: [&'static str; 6] = ["en", "zh-TW", "ja", "zh-Hant", "ko", "zh"];

async fn get_transcript(video: &str) -> String {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(120))
        .build().unwrap();
    let digest = DigestScraper::new(client);

    // Fetch the video
    let scraped = fetch_video(video, digest).await;

    // scraped.captions.iter().for_each(|caption| println!("{}", caption.lang_tag));

    // Find our preferred language, the priority is the order of LANGUAGES
    let language = get_caption_language(&scraped).unwrap();
    let captions = scraped.captions.iter().find(|caption| caption.lang_tag == language).unwrap();

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

fn get_caption_language(scraped: &Digest) -> Option<LanguageTag> {
    for lang in LANGUAGES.iter() {
        let language = LanguageTag::parse(lang).unwrap();
        if scraped.captions.iter().any(|caption| language.matches(&caption.lang_tag)) {
            return Some(language);
        }
    }
    None
}

fn find_preferred_language() -> Option<LanguageTag> {
    let mut language = None;

    for lang in LANGUAGES {
        match LanguageTag::parse(lang) {
            Ok(result) => {
                language = Some(result);
                break;
            }
            Err(_) => continue,
        }
    }
    language
}

async fn fetch_video(video: &str, digest: DigestScraper) -> Digest {
    let mut scraped = None;

    for lang in LANGUAGES {
        match digest.fetch(video, lang).await {
            Ok(result) => {
                scraped = Some(result);
                break;
            }
            Err(_) => continue,
        }
    }

    let scraped = scraped.unwrap();
    scraped
}

pub(crate) async fn summarize_video(video: &str, openai_api_key: &str, lang: &str) -> String {
    let client = Client::with_config(
        OpenAIConfig::default(),
    );

    //First, we fetch the transcript for the video
    let transcript = get_transcript(video).await;

    // Then we create our summary agent and have it summarize the video for us

    let mut summarize_agent = Agent {
        system: get_summary_prompt(lang),
        model: "gpt-4o".to_string(),
        history: vec![],
        client: client.clone(),
    };

    let mut summary_to_json_agent = Agent {
        system: get_summary_to_json_prompt(lang),
        model: "gpt-4o".to_string(),
        history: vec![],
        client: client.clone(),
    };

    let summary = summarize_agent.prompt(transcript, openai_api_key).await.unwrap();

    let json = summary_to_json_agent.prompt(summary, openai_api_key).await.unwrap();
    let result = Agent::extract_codeblock(&json);

    result
}



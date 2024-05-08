use async_openai::Client;
use async_openai::config::OpenAIConfig;
use async_openai::error::OpenAIError;
use async_openai::types::{ChatCompletionRequestMessage, ChatCompletionRequestSystemMessageArgs, ChatCompletionRequestUserMessageArgs, CreateChatCompletionRequestArgs};
use crate::Agent;

static SUMMARY_PROMPT : &str = r#"You are an agent dedicated to summarising video transcripts.
You will receive a transcript and answer with main talking points of the video fi()rst,
followed by a complete summary of the transcript. Answer only in this format and translate to $:


Talking points:
1. ..
2. ..
N. ..

Summary:
Summary of the transcript
"#;

static SUMMARY_TO_JSON_PROMPT: &str = r#"You are an agent dedicated to translating text to JSON. You will receive the text and return it in JSON format.
The format is as follows and transalte JSON value to $ in the:


{

    “summary”: “Whole video summary goes here”,
   “talking_points”: [
{
   “title” : “Title of the point”,
   “description: “Talking point summary”
 },
...
]
}

Rules:
- Follow the specified JSON format closely
- Wrap the JSON in a code block
- Skip prose, return only the JSON
- Only translate the JSON value, do not translate the keys
"#;

pub fn get_summary_prompt(lang: &str) -> String {
    SUMMARY_PROMPT.replace("$", lang)
}

pub fn get_summary_to_json_prompt(lang: &str) -> String {
    SUMMARY_TO_JSON_PROMPT.replace("$", lang)
}

impl Agent {
    pub(crate) async fn prompt(&mut self, input: String, openai_api_key: &str) -> Result<String, OpenAIError> {
        let config = OpenAIConfig::new()
            .with_api_key(openai_api_key);

        let client = Client::with_config(config);

        client.chat().create(
            CreateChatCompletionRequestArgs::default()
                .model(self.model.clone())
                .messages(vec![

                    //First we add the system message to define what the Agent does
                    ChatCompletionRequestMessage::System(
                        ChatCompletionRequestSystemMessageArgs::default()
                            .content(&self.system)
                            .build()
                            .unwrap(),
                    ),

                    //Then we add our prompt
                    ChatCompletionRequestMessage::User(
                        ChatCompletionRequestUserMessageArgs::default()
                            .content(input)
                            .build()
                            .unwrap(),
                    ),
                ])
                .build()
                .unwrap(),
        ).await.map(|res| {
            //We extract the first one
            res.choices[0].message.content.clone().unwrap()
        })

        //Now here, you can save the prompt and agent response to the history if needed
    }

    pub(crate) fn extract_codeblock(text: &str) -> String {
        if !text.contains("```") {
            return text.to_string();
        }
        let mut in_codeblock = false;
        let mut extracted_lines = vec![];

        for line in text.lines() {
            if line.trim().starts_with("```") {
                in_codeblock = !in_codeblock;
                continue;
            }

            if in_codeblock {
                extracted_lines.push(line);
            }
        }

        extracted_lines.join("\n")
    }

}


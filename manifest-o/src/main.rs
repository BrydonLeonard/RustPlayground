use reqwest::header;
use std::env;
use std::fs;
use arg_parsing::Args;
use open_ai::*;

const OPENAI_ENDPOINT: &str = "https://api.openai.com/v1/chat/completions";

fn main() -> Result<(), &'static str> {
    let args = Args::build(env::args().into_iter())?;
    let file_contents: String =
        fs::read_to_string(&args.file_path).expect("Failed to read file contents");

    let client = build_openai_client(&args);

    let manifesto_summary = get_manifesto_summary(&client, &file_contents)
        .expect("Failed to summarise manifesto");

    println!("{}", manifesto_summary);

    Ok(())
}

fn build_openai_client(args: &Args) -> reqwest::blocking::Client {
    let mut headers = reqwest::header::HeaderMap::new();

    let header_value: String = format!("Bearer {}", args.openai_key);

    let header_value = header::HeaderValue::from_str(&header_value)
        .expect("Couldn't build header with OpenAI key");

    headers.insert(header::AUTHORIZATION, header_value);

    reqwest::blocking::Client::builder()
        .default_headers(headers)
        .build()
        .expect("Failed to build OpenAI client")
}

fn get_manifesto_summary(client: &reqwest::blocking::Client, manifesto: &str) -> Result<String, &'static str> {

    let req = OpenAiRequestBody {
        model: GPT_35_MODEL_NAME,
        messages: vec![
            OpenAiRequestMessage {
                role: "system",
                content: "You are an experienced political journalist that writes four-paragraph summaries of the manifestos of political parties"
            },
            OpenAiRequestMessage {
                role: "user",
                content: "Please summarise the following manifesto:"
            },
            OpenAiRequestMessage {
                role: "user", 
                content: &manifesto
            }
        ]
    };


    let resp: OpenAiResponse = client
        .post(OPENAI_ENDPOINT)
        .json(&req)
        .send()
        .expect("Couldn't make request")
        .json()
        .expect("Couldn't deserialize as JSON");

    Ok(format!("{}", resp))
}

mod open_ai {
    use serde::{ Serialize, Deserialize };
    use std::fmt;

    pub const GPT_35_MODEL_NAME: &str = "gpt-3.5-turbo";
    pub const _GPT_4_MODEL_NAME: &str = "gpt-4-turbo";

    #[derive(Serialize)]
    pub struct OpenAiRequestBody<'a> {
        pub model: &'a str,
        pub messages: Vec<OpenAiRequestMessage<'a>>,
    }

    #[derive(Serialize)]
    pub struct OpenAiRequestMessage<'a> {
        pub role: &'a str,
        pub content: &'a str,
    }

    #[derive(Deserialize)]
    pub struct OpenAiResponse {
        pub choices: Vec<OpenAiResponseMessage>,
    }

    impl fmt::Display for OpenAiResponse {
        fn fmt(&self, formatter: &mut fmt::Formatter) -> Result<(), fmt::Error> {
            write!(formatter, "{}", self.choices[0].message.content)?;

            Ok(())
        }
    }

    #[derive(Deserialize)]
    pub struct OpenAiResponseMessage {
        pub message: OpenAiResponseMessageContent,
    }

    #[derive(Deserialize)]
    pub struct OpenAiResponseMessageContent {
        pub content: String
    }
}

mod arg_parsing {
    use std::fs;

    pub struct Args {
        pub file_path: String,
        pub openai_key: String,
    }

    impl Args {
        pub fn build(mut args: impl Iterator<Item = String>) -> Result<Args, &'static str> {
            args.next(); // First arg is the executable's name

            let file_path = match args.next() {
                Some(arg) => arg,
                None => return Err("Didn't get a file_path"),
            };

            let openai_key_file_path = match args.next() {
                Some(arg) => arg,
                None => return Err("Didn't get a file path for the OpenAI key"),
            };

            let mut openai_key: String =
                fs::read_to_string(openai_key_file_path).expect("Failed to read OpenAI key file");

            if openai_key.ends_with('\n') {
                openai_key.pop();
            }

            Ok(Args {
                file_path,
                openai_key,
            })
        }
    }
}

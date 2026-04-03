use async_openai::{Client, config::OpenAIConfig};
use clap::Parser;
use serde_json::{Value, json};
use std::{env, process};

#[derive(Parser)]
#[command(author, version, about)]
struct Args {
    #[arg(short = 'p', long)]
    prompt: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let base_url = env::var("OPENROUTER_BASE_URL")
        .unwrap_or_else(|_| "https://openrouter.ai/api/v1".to_string());

    let api_key = env::var("OPENROUTER_API_KEY").unwrap_or_else(|_| {
        eprintln!("OPENROUTER_API_KEY is not set");
        process::exit(1);
    });

    let config = OpenAIConfig::new()
        .with_api_base(base_url)
        .with_api_key(api_key);

    let client = Client::with_config(config);

    let mut messages: Vec<Value> = vec![json!({"role": "user", "content": args.prompt})];

    loop {
        #[allow(unused_variables)]
        let response: Value = client
            .chat()
            .create_byot(json!({
                        "messages": messages,
                        "model": "anthropic/claude-3.5-haiku",
                        "tools": [
                        {
                          "type": "function",
                          "function": {
                            "name": "Read",
                            "description": "Read and return the contents of a file",
                            "parameters": {
                              "type": "object",
                              "properties": {
                                "file_path": {
                                  "type": "string",
                                  "description": "The path to the file to read"
                                }
                              },
                              "required": ["file_path"]
                            }
                          }
                        },
            ]
                    }))
            .await?;
        // You can use print statements as follows for debugging, they'll be visible when running tests.
        let message = &response["choices"][0]["message"];

        // TODO: Uncomment the lines below to pass the first stage
        if let Some(content) = response["choices"][0]["message"]["content"].as_str() {
            println!("{}", content);
        }

        if let Some(tools_call) = response["choices"][0]["message"]["tool_calls"].as_array() {
            eprintln!("Logs from your program will appear here!");

            messages.push(message.clone());

            for tool in tools_call {
                let argument: Value =
                    serde_json::from_str(tool["function"]["arguments"].as_str().unwrap())?;
                if let Some(tool_name) = tool["function"]["name"].as_str() {
                    if tool_name == "Read" {
                        let file_path = argument["file_path"].as_str().unwrap();
                        let content = std::fs::read_to_string(file_path)?;
                        messages.push(json!(
                            {
                                "role": "tool",
                                "tool_call_id": tool["id"].as_str().unwrap(),
                                "name": tool_name,
                                "content": content
                            }
                        ));
                    }
                }
            }
            continue;
        }
        break;
    }
    Ok(())
}

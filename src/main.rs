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
                        "model": "anthropic/claude-haiku-4.5",
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
                        {
                          "type": "function",
                          "function": {
                            "name": "Write",
                            "description": "Write content to a file",
                            "parameters": {
                              "type": "object",
                              "required": ["file_path", "content"],
                              "properties": {
                                "file_path": {
                                  "type": "string",
                                  "description": "The path of the file to write to"
                                },
                                "content": {
                                  "type": "string",
                                  "description": "The content to write to the file"
                                }
                              }
                            }
                          }
                        },
            ]
                    }))
            .await?;
        // You can use print statements as follows for debugging, they'll be visible when running tests.
        let message = &response["choices"][0]["message"];
        let has_tools_call = message["tool_calls"].as_array().is_some();

        // TODO: Uncomment the lines below to pass the first stage
        if !has_tools_call {
            if let Some(content) = response["choices"][0]["message"]["content"].as_str() {
                println!("{}", content);
            }
        }

        if let Some(tools_call) = response["choices"][0]["message"]["tool_calls"].as_array() {
            eprintln!("Logs from your program will appear here!");

            messages.push(message.clone());

            for tool in tools_call {
                let argument: Value =
                    serde_json::from_str(tool["function"]["arguments"].as_str().unwrap())?;
                if let Some(tool_name) = tool["function"]["name"].as_str() {
                    match tool_name {
                        "Read" => {
                            let file_path = argument["file_path"].as_str().unwrap();
                            let content = match std::fs::read_to_string(file_path) {
                                Ok(c) => c,
                                Err(e) => format!("Error reading file: {}", e),
                            };

                            messages.push(json!({
                                "role":"tool",
                                "tool_call_id":tool["id"].as_str().unwrap(),
                                "content":content,

                            }));
                        }
                        "Write" => {
                            let file_path = argument["file_path"].as_str().unwrap();
                            let file_content = argument["content"].as_str().unwrap();
                            let content = match std::fs::write(file_path, file_content) {
                                Ok(_) => "File written successfully".to_string(),
                                Err(e) => format!("Error writing file: {e}"),
                            };

                            messages.push(json!({
                                "role":"tool",
                                "tool_call_id":tool["id"].as_str().unwrap(),
                                "content":content,
                            }
                            ));
                        }
                        _ => println!("hello"),
                    }
                }
            }
            continue;
        }
        break;
    }
    Ok(())
}

use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::error::Error;

use crate::database;

#[derive(Serialize)]
struct Content {
    parts: Vec<Part>,
    role: String,
}

#[derive(Serialize)]
struct Part {
    text: String,
}

#[derive(Serialize)]
struct RequestBody {
    contents: Vec<Content>,
}

#[derive(Deserialize, Debug)]
struct CandidatePart {
    text: String,
}

#[derive(Deserialize, Debug)]
struct Candidate {
    content: CandidateContent,
}

#[derive(Deserialize, Debug)]
struct CandidateContent {
    parts: Vec<CandidatePart>,
}

#[derive(Deserialize, Debug)]
struct GeminiResponse {
    candidates: Vec<Candidate>,
}

pub async fn ask_gemini(prompt: String) -> Result<String, Box<dyn std::error::Error>> {
    // Get database connection
    let db =
        database::DBtodo::new().map_err(|e| format!("Failed to connect to database: {}", e))?;

    // Get todos for context
    let todos = db
        .get_todos()
        .map_err(|e| format!("Failed to get todos: {}", e))?;

    // Get API key
    let api_key = db.get_api_credentials().map_err(|e| {
        format!(
            "Failed to get API credentials: {}. Did you set an API key using the -k flag?",
            e
        )
    })?;

    if api_key.is_empty() {
        return Err("No API key found. Please set one using the -k flag first.".into());
    }

    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.0-flash:generateContent?key={}",
        api_key
    );

    // Format todos as text for the prompt
    let todos_text = todos
        .iter()
        .map(|todo| {
            format!(
                "- [{}] {} (Priority: {}, Due: {}, Description: {}, Subtasks: {:?}, Owner: {}, Topic: {})",
                if todo.status == "Done" { "x" } else { " " },
                todo.text,
                todo.priority,
                todo.due,
                todo.desc,
                todo.subtasks,
                todo.owner,
                todo.topic
            )
        })
        .collect::<Vec<String>>()
        .join("\n");

    // Create system prompt with instructions and context
    let system_prompt = format!(
        "ROLE: You are an AI assistant for a todo application.
RULES:
- Respond in clear markdown formatting
- Be concise and actionable
- Reference existing todos when relevant
- Today is {}
- Current time: {}

CURRENT TODOS:
{}

USER REQUEST: {}",
        chrono::Local::now().format("%A, %B %d"),
        chrono::Local::now().format("%H:%M"),
        todos_text,
        prompt
    );

    let client = reqwest::Client::new();
    let body = RequestBody {
        contents: vec![Content {
            role: "user".to_string(),
            parts: vec![Part {
                text: system_prompt,
            }],
        }],
    };

    let response = client
        .post(&url)
        .json(&body)
        .send()
        .await?
        .error_for_status()?
        .json::<GeminiResponse>()
        .await?;

    if let Some(candidate) = response.candidates.first() {
        if let Some(part) = candidate.content.parts.first() {
            return Ok(part.text.clone());
        }
    }

    Ok("❌ No response from Gemini.".into())
}

use async_openai::types::{CreateChatCompletionRequestArgs, ChatCompletionRequestSystemMessageArgs, ChatCompletionRequestUserMessageArgs};
use async_openai::Client as LLM_Client;
use anyhow::Result;
use async_openai::types::ResponseFormat::JsonObject;
use crate::parser::RequestStructure;
use serde_json::{json, Value};
use tracing::info;

pub(crate) async fn llm_engine(system_role: String, request: String) -> Result<String> {
    let client = LLM_Client::new();

    let llm_request = CreateChatCompletionRequestArgs::default()
        .max_tokens(8192u32)
        .model("gpt-4o-2024-08-06")
        .temperature(0.1)
        .messages([
            ChatCompletionRequestSystemMessageArgs::default()
                .content(system_role.as_str())
                .build()?
                .into(),
            ChatCompletionRequestUserMessageArgs::default()
                .content(request)
                .build()?
                .into(),
        ])
        .build()?;
    
    let response = client.chat().create(llm_request).await?;
    
    if let Some(choice) = response.choices.get(0) {
        let content = choice.message.content.clone().unwrap_or_else(|| {
            "Извини, я не смог понять твой вопрос. Пожалуйста, попробуй снова.".to_string()
        });
        Ok(content)
    } else {
        Ok("Извини, я не смог понять твой вопрос. Пожалуйста, попробуй снова.".to_string())
    }
}

pub(crate) async fn llm_engine_json(system_role: String, request: String) -> Result<String> {
    let client = LLM_Client::new();

    let llm_request = CreateChatCompletionRequestArgs::default()
        .max_tokens(8192u32)
        .model("gpt-4o-2024-08-06")
        .temperature(0.1)
        .messages([
            ChatCompletionRequestSystemMessageArgs::default()
                .content(system_role.as_str())
                .build()?
                .into(),
            ChatCompletionRequestUserMessageArgs::default()
                .content(request)
                .build()?
                .into(),
        ])
        .response_format(JsonObject)
        .build()?;

    let response = client.chat().create(llm_request).await?;

    if let Some(choice) = response.choices.get(0) {
        let content = choice.message.content.clone().unwrap_or_else(|| {
            "Извини, я не смог понять твой вопрос. Пожалуйста, попробуй снова.".to_string()
        });
        Ok(content)
    } else {
        Ok("Извини, я не смог понять твой вопрос. Пожалуйста, попробуй снова.".to_string())
    }
}

pub(crate) async fn core_llm_engine(
    system_role: String,
    request_structure: &RequestStructure,
) -> Result<String> {
    let request_json = serde_json::to_string_pretty(request_structure)?;
    
    let llm_payload = json!({
        "system_role": system_role,
        "request_structure": request_json
    });

    // Предположим, что у нас есть функция `llm_engine_json`, которая отправляет запрос
    let llm_response = llm_engine(system_role, llm_payload.to_string()).await?;
    
    info!("llm_final_response: {}", llm_response);
    
    Ok(llm_response)
}

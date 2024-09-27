use crate::ai_utils::{core_llm_engine, llm_engine, llm_engine_json};
use crate::parser::{update_request_structure, RequestStructure, UserProfile};
use chrono::Local;
use serde_json::Value;
use std::collections::{HashMap, VecDeque};
use std::fs;
use std::sync::Arc;
use teloxide::prelude::{ChatId, Message, Requester};
use teloxide::Bot;
use tokio::sync::Mutex;
use tracing::info;

pub(crate) async fn message_handler(
    bot: Bot,
    msg: Message,
    app_state: Arc<AppState>,
) -> anyhow::Result<()> {
    create_structured_request(bot, msg, app_state).await?;

    Ok(())
}

pub(crate) async fn create_structured_request(
    bot: Bot,
    msg: Message,
    app_state: Arc<AppState>,
) -> anyhow::Result<()> {
    let user_request = msg.text().unwrap().to_string();

    let mut user_states = app_state.user_state.lock().await;

    let user_state = user_states.entry(msg.chat.id).or_insert_with(|| UserState {
        dialogue_cache: DialogueCache::new(),
    });

    user_state
        .dialogue_cache
        .add_user_message(user_request.clone());

    let mut request_structure = RequestStructure {
        request: String::new(),
        cache: vec![],
        context: String::new(),
        viewpoints: vec![],
        user_profile: UserProfile {
            expertise_lvl: String::new(),
            communication_style: String::new(),
        },
    };

    // Step 1. Updating structure with user_request
    update_request_structure(&mut request_structure, "request", Some(&user_request)).await;

    // Check if we have proper JSON-structure
    let json_output_1 = serde_json::to_string_pretty(&request_structure).unwrap();
    info!("JSON str. after updating request: {}", json_output_1);

    // Step 1.5. Updating structure with cache
    let dialogue_context = user_state.dialogue_cache.to_string();
    update_request_structure(&mut request_structure, "cache", Some(&dialogue_context)).await;

    // Step 2. Updating structure with context
    let system_role_for_context = fs::read_to_string("common_res/system_role_for_context.txt")
        .map_err(|e| format!("Failed to read 'system role': {}", e))
        .unwrap();
    let llm_response =
        llm_engine(system_role_for_context.to_string(), user_request.clone()).await?;

    update_request_structure(&mut request_structure, "context", Some(&llm_response)).await;

    // Check if we have proper JSON-structure
    let json_output_2 = serde_json::to_string_pretty(&request_structure).unwrap();
    info!("JSON str. after adding context: {}", json_output_2);

    // Step 3. Updating structure with viewpoints array

    let system_role_for_viewpoints =
        fs::read_to_string("common_res/system_role_for_viewpoints.txt")
            .map_err(|e| format!("Failed to read 'system role': {}", e))
            .unwrap();
    let llm_response =
        llm_engine_json(system_role_for_viewpoints.to_string(), user_request.clone()).await?;

    let parsed_response: Value = serde_json::from_str(&llm_response)?;
    if let Some(viewpoints_array) = parsed_response.get("viewpoints") {
        if let Some(viewpoints_vec) = viewpoints_array.as_array() {
            // Extracting strings from the array and generate the required JSON
            let viewpoints: Vec<String> = viewpoints_vec
                .iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string())) // Extracting strings from an array
                .collect();
            // Form a string in the required format
            let generated_viewpoints =
                serde_json::to_string(&viewpoints).unwrap_or_else(|_| r#"[]"#.to_string());

            //  Updating structure with viewpoints
            update_request_structure(
                &mut request_structure,
                "viewpoints",
                Some(&generated_viewpoints),
            )
            .await;
        }
    }

    // Check if we have proper JSON-structure
    let json_output_3 = serde_json::to_string_pretty(&request_structure).unwrap();
    info!("JSON str. after adding viewpoints: {}", json_output_3);

    // Step 4. Updating structure with user_profile (expertise level and communication style)
    let system_role_for_user_profile =
        fs::read_to_string("common_res/system_role_for_user_profile.txt")
            .map_err(|e| format!("Failed to read 'system role': {}", e))
            .unwrap();
    let llm_response = llm_engine_json(
        system_role_for_user_profile.to_string(),
        user_request.clone(),
    )
    .await?;

    let parsed_response: Value = serde_json::from_str(&llm_response)?;
    if let Some(expertise) = parsed_response
        .get("expertise_lvl")
        .and_then(|v| v.as_str())
    {
        update_request_structure(&mut request_structure, "expertise_lvl", Some(expertise)).await;
    }
    if let Some(style) = parsed_response
        .get("communication_style")
        .and_then(|v| v.as_str())
    {
        update_request_structure(&mut request_structure, "communication_style", Some(style)).await;
    }

    // Check if we have proper JSON-structure
    let json_output_4 = serde_json::to_string_pretty(&request_structure).unwrap();
    info!("Final JSON after adding user_profile: {}", json_output_4);

    // Step 5. Updating structure with cache

    let dialogue_context = user_state.dialogue_cache.to_vec();
    info!("cache: {:?}", dialogue_context);
    update_request_structure(
        &mut request_structure,
        "cache",
        Some(&serde_json::to_string(&dialogue_context)?),
    )
    .await;

    let json_output_5 = serde_json::to_string_pretty(&request_structure).unwrap();
    info!("Final JSON after adding user_profile: {}", json_output_5);

    let final_response =
        core_llm_engine("Ответь на запрос".to_string(), &request_structure).await?;
    user_state
        .dialogue_cache
        .update_last_response(final_response.clone());
    bot.send_message(msg.chat.id, final_response).await?;
    Ok(())
}

pub struct AppState {
    pub(crate) user_state: Mutex<HashMap<ChatId, UserState>>,
}

#[derive(Default)]
pub struct UserState {
    dialogue_cache: DialogueCache,
}

#[derive(Debug)]
pub struct UserInteraction {
    timestamp: String,
    user_request: String,
    llm_response: String,
}

#[derive(Default)]
pub struct DialogueCache {
    messages: VecDeque<UserInteraction>,
    max_size: usize,
}

impl DialogueCache {
    fn new() -> Self {
        DialogueCache {
            messages: VecDeque::new(),
            max_size: 10,
        }
    }

    fn add_user_message(&mut self, user_question: String) {
        let now = Local::now();
        let timestamp = now.format("%Y-%m-%d %H:%M:%S (%A)").to_string();

        let entry = UserInteraction {
            timestamp,
            user_request: user_question,
            llm_response: String::new(),
        };

        self.messages.push_back(entry);
        if self.messages.len() > self.max_size {
            self.messages.pop_front();
        }
    }

    fn update_last_response(&mut self, llm_response: String) {
        if let Some(last_interaction) = self.messages.back_mut() {
            last_interaction.llm_response = llm_response;
        }
    }

    fn to_string(&self) -> String {
        self.messages
            .iter()
            .map(|interaction| {
                format!(
                    "[{}] User: {}\nAssistant: {}",
                    interaction.timestamp, interaction.user_request, interaction.llm_response
                )
            })
            .collect::<Vec<_>>()
            .join("\n\n")
    }
    fn to_vec(&self) -> Vec<String> {
        self.messages
            .iter()
            .map(|interaction| {
                format!(
                    "[{}] User: {}\nAssistant: {}",
                    interaction.timestamp, interaction.user_request, interaction.llm_response
                )
            })
            .collect::<Vec<String>>()
    }
}

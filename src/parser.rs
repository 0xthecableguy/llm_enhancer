use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct RequestStructure {
    pub request: String,
    pub cache: Vec<String>,
    pub context: String,
    pub viewpoints: Vec<String>,
    pub user_profile: UserProfile,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UserProfile {
    pub expertise_lvl: String,
    pub communication_style: String,
}

pub(crate) async fn update_request_structure(
    request_structure: &mut RequestStructure,
    field: &str,
    value: Option<&str>,
) {
    match field {
        "request" => {
            if let Some(val) = value {
                request_structure.request = val.to_string();
            }
        }
        "cache" => {
            if let Some(val) = value {
                let cache: Vec<String> = serde_json::from_str(val).unwrap_or_else(|_| vec![]);
                request_structure.cache = cache;
            }
        }
        "context" => {
            if let Some(val) = value {
                request_structure.context = val.to_string();
            }
        }
        "viewpoints" => {
            if let Some(val) = value {
                let vp: Vec<String> = serde_json::from_str(val).unwrap_or_else(|_| vec![]);
                request_structure.viewpoints = vp;
            }
        }
        "expertise_lvl" => {
            if let Some(val) = value {
                request_structure.user_profile.expertise_lvl = val.to_string();
            }
        }
        "communication_style" => {
            if let Some(val) = value {
                request_structure.user_profile.communication_style = val.to_string();
            }
        }
        _ => {
            log::warn!("Unknown field: {}", field);
        }
    }
}

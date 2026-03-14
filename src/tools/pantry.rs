use crate::tools::traits::{Tool, ToolResult};
use crate::security::SecurityPolicy;
use anyhow::{Result, Context};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use chrono::{DateTime, Utc, Duration, NaiveDate};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PantryItem {
    pub id: String,
    pub name: String,
    pub quantity: String,
    pub purchase_date: String,
    pub expiry_date: String,
}

pub struct PantryTool {
    security: Arc<SecurityPolicy>,
    workspace_dir: PathBuf,
}

impl PantryTool {
    pub fn new(security: Arc<SecurityPolicy>, workspace_dir: PathBuf) -> Self {
        Self { security, workspace_dir }
    }

    fn pantry_path(&self) -> PathBuf {
        self.workspace_dir.join("pantry.json")
    }

    fn load_pantry(&self) -> Result<Vec<PantryItem>> {
        let path = self.pantry_path();
        if !path.exists() {
            return Ok(Vec::new());
        }
        let content = std::fs::read_to_string(&path)?;
        let items = serde_json::from_str(&content).unwrap_or_default();
        Ok(items)
    }

    fn save_pantry(&self, items: &[PantryItem]) -> Result<()> {
        let path = self.pantry_path();
        let content = serde_json::to_string_pretty(items)?;
        std::fs::write(path, content)?;
        Ok(())
    }
}

#[async_trait]
impl Tool for PantryTool {
    fn name(&self) -> &str {
        "pantry"
    }

    fn description(&self) -> &str {
        "Manage pantry items and expiry dates. Actions: add, list, remove, check_expiring."
    }

    fn parameters_schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "action": {
                    "type": "string",
                    "enum": ["add", "list", "remove", "check_expiring"],
                    "description": "The action to perform."
                },
                "items": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "name": { "type": "string" },
                            "quantity": { "type": "string", "default": "1" },
                            "expiry_date": { "type": "string", "description": "YYYY-MM-DD" }
                        },
                        "required": ["name", "expiry_date"]
                    },
                    "description": "List of items to add (required for 'add')."
                },
                "item_id": {
                    "type": "string",
                    "description": "The ID of the item to remove (required for 'remove')."
                },
                "days_threshold": {
                    "type": "integer",
                    "default": 7,
                    "description": "Days remaining to consider an item as 'expiring' (used in 'check_expiring')."
                }
            },
            "required": ["action"]
        })
    }

    async fn execute(&self, args: serde_json::Value) -> Result<ToolResult> {
        let action = args["action"].as_str().context("action is required")?;
        let mut pantry = self.load_pantry()?;

        match action {
            "add" => {
                let items_to_add = args["items"].as_array().context("items array is required for 'add'")?;
                let now = Utc::now().format("%Y-%m-%d").to_string();
                
                for item_val in items_to_add {
                    let name = item_val["name"].as_str().context("item name is required")?;
                    let quantity = item_val["quantity"].as_str().unwrap_or("1");
                    let expiry = item_val["expiry_date"].as_str().context("expiry_date is required")?;
                    
                    pantry.push(PantryItem {
                        id: uuid::Uuid::new_v4().to_string(),
                        name: name.to_string(),
                        quantity: quantity.to_string(),
                        purchase_date: now.clone(),
                        expiry_date: expiry.to_string(),
                    });
                }
                self.save_pantry(&pantry)?;
                Ok(ToolResult::success(format!("Successfully added {} items.", items_to_add.len())))
            }
            "list" => {
                let output = serde_json::to_string_pretty(&pantry)?;
                Ok(ToolResult::success(output))
            }
            "remove" => {
                let id = args["item_id"].as_str().context("item_id is required for 'remove'")?;
                let initial_len = pantry.len();
                pantry.retain(|i| i.id != id);
                if pantry.len() < initial_len {
                    self.save_pantry(&pantry)?;
                    Ok(ToolResult::success(format!("Item {} removed.", id)))
                } else {
                    Ok(ToolResult::error(format!("Item {} not found.", id)))
                }
            }
            "check_expiring" => {
                let threshold = args["days_threshold"].as_i64().unwrap_or(7);
                let now = Utc::now().naive_utc().date();
                let limit = now + Duration::days(threshold);
                
                let expiring: Vec<PantryItem> = pantry.iter().filter(|i| {
                    if let Ok(expiry_date) = NaiveDate::parse_from_str(&i.expiry_date, "%Y-%m-%d") {
                        expiry_date <= limit
                    } else {
                        false
                    }
                }).cloned().collect();
                
                let output = serde_json::to_string_pretty(&expiring)?;
                Ok(ToolResult::success(output))
            }
            _ => Ok(ToolResult::error(format!("Unknown action: {}", action))),
        }
    }
}

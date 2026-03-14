use crate::tools::traits::{Tool, ToolResult};
use crate::security::SecurityPolicy;
use anyhow::{Result, Context};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::path::{Path, PathBuf};
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InventoryItem {
    pub id: String,
    pub name: String,
    pub location: String,
    pub updated_at: String,
}

pub struct InventoryTool {
    security: Arc<SecurityPolicy>,
    workspace_dir: PathBuf,
}

impl InventoryTool {
    pub fn new(security: Arc<SecurityPolicy>, workspace_dir: PathBuf) -> Self {
        Self { security, workspace_dir }
    }

    fn inventory_path(&self) -> PathBuf {
        self.workspace_dir.join("inventory.json")
    }

    fn load_inventory(&self) -> Result<Vec<InventoryItem>> {
        let path = self.inventory_path();
        if !path.exists() {
            return Ok(Vec::new());
        }
        let content = std::fs::read_to_string(&path)?;
        let items = serde_json::from_str(&content).unwrap_or_default();
        Ok(items)
    }

    fn save_inventory(&self, items: &[InventoryItem]) -> Result<()> {
        let path = self.inventory_path();
        let content = serde_json::to_string_pretty(items)?;
        std::fs::write(path, content)?;
        Ok(())
    }
}

#[async_trait]
impl Tool for InventoryTool {
    fn name(&self) -> &str {
        "inventory"
    }

    fn description(&self) -> &str {
        "Track locations of household items (tools, documents, etc.). Actions: add, find, list, remove."
    }

    fn parameters_schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "action": {
                    "type": "string",
                    "enum": ["add", "find", "list", "remove"],
                    "description": "The action to perform."
                },
                "name": {
                    "type": "string",
                    "description": "Name of the item (required for 'add', 'find', 'remove')."
                },
                "location": {
                    "type": "string",
                    "description": "Location where the item is stored (required for 'add')."
                }
            },
            "required": ["action"]
        })
    }

    async fn execute(&self, args: serde_json::Value) -> Result<ToolResult> {
        let action = args["action"].as_str().context("action is required")?;
        let mut inventory = self.load_inventory()?;

        match action {
            "add" => {
                let name = args["name"].as_str().context("name is required for 'add'")?;
                let location = args["location"].as_str().context("location is required for 'add'")?;
                let now = chrono::Utc::now().to_rfc3339();
                
                // Update existing item if name matches (case-insensitive)
                let mut found = false;
                for item in &mut inventory {
                    if item.name.eq_ignore_ascii_case(name) {
                        item.location = location.to_string();
                        item.updated_at = now.clone();
                        found = true;
                        break;
                    }
                }

                if !found {
                    inventory.push(InventoryItem {
                        id: uuid::Uuid::new_v4().to_string(),
                        name: name.to_string(),
                        location: location.to_string(),
                        updated_at: now,
                    });
                }
                
                self.save_inventory(&inventory)?;
                Ok(ToolResult::success(format!("Successfully registered: {} is in {}.", name, location)))
            }
            "find" => {
                let name = args["name"].as_str().context("name is required for 'find'")?;
                let matches: Vec<InventoryItem> = inventory.iter()
                    .filter(|i| i.name.to_lowercase().contains(&name.to_lowercase()))
                    .cloned()
                    .collect();
                
                if matches.is_empty() {
                    Ok(ToolResult::success(format!("I couldn't find '{}' in the inventory.", name)))
                } else {
                    let output = serde_json::to_string_pretty(&matches)?;
                    Ok(ToolResult::success(output))
                }
            }
            "list" => {
                let output = serde_json::to_string_pretty(&inventory)?;
                Ok(ToolResult::success(output))
            }
            "remove" => {
                let name = args["name"].as_str().context("name is required for 'remove'")?;
                let initial_len = inventory.len();
                inventory.retain(|i| !i.name.eq_ignore_ascii_case(name));
                
                if inventory.len() < initial_len {
                    self.save_inventory(&inventory)?;
                    Ok(ToolResult::success(format!("Item '{}' removed from inventory.", name)))
                } else {
                    Ok(ToolResult::error(format!("Item '{}' not found.", name)))
                }
            }
            _ => Ok(ToolResult::error(format!("Unknown action: {}", action))),
        }
    }
}

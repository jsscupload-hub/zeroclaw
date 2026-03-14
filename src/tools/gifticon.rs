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
pub struct Gifticon {
    pub id: String,
    pub name: String,
    pub expiry_date: String,
    pub status: String, // "available" or "used"
}

pub struct GifticonTool {
    security: Arc<SecurityPolicy>,
    workspace_dir: PathBuf,
}

impl GifticonTool {
    pub fn new(security: Arc<SecurityPolicy>, workspace_dir: PathBuf) -> Self {
        Self { security, workspace_dir }
    }

    fn gifticons_path(&self) -> PathBuf {
        self.workspace_dir.join("gifticons.json")
    }

    fn load_gifticons(&self) -> Result<Vec<Gifticon>> {
        let path = self.gifticons_path();
        if !path.exists() {
            return Ok(Vec::new());
        }
        let content = std::fs::read_to_string(&path)?;
        let items = serde_json::from_str(&content).unwrap_or_default();
        Ok(items)
    }

    fn save_gifticons(&self, items: &[Gifticon]) -> Result<()> {
        let path = self.gifticons_path();
        let content = serde_json::to_string_pretty(items)?;
        std::fs::write(path, content)?;
        Ok(())
    }
}

#[async_trait]
impl Tool for GifticonTool {
    fn name(&self) -> &str {
        "gifticon"
    }

    fn description(&self) -> &str {
        "Manage gifticons and vouchers. Actions: add, list, use, check_expiring."
    }

    fn parameters_schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "action": {
                    "type": "string",
                    "enum": ["add", "list", "use", "check_expiring"],
                    "description": "The action to perform."
                },
                "name": {
                    "type": "string",
                    "description": "Name of the gifticon (required for 'add')."
                },
                "expiry_date": {
                    "type": "string",
                    "description": "Expiry date YYYY-MM-DD (required for 'add')."
                },
                "gifticon_id": {
                    "type": "string",
                    "description": "ID of the gifticon to use (required for 'use')."
                },
                "days_threshold": {
                    "type": "integer",
                    "default": 7,
                    "description": "Days remaining to notify (used in 'check_expiring')."
                }
            },
            "required": ["action"]
        })
    }

    async fn execute(&self, args: serde_json::Value) -> Result<ToolResult> {
        let action = args["action"].as_str().context("action is required")?;
        let mut gifticons = self.load_gifticons()?;

        match action {
            "add" => {
                let name = args["name"].as_str().context("name is required for 'add'")?;
                let expiry = args["expiry_date"].as_str().context("expiry_date is required for 'add'")?;
                
                let id = uuid::Uuid::new_v4().to_string();
                gifticons.push(Gifticon {
                    id: id.clone(),
                    name: name.to_string(),
                    expiry_date: expiry.to_string(),
                    status: "available".to_string(),
                });
                
                self.save_gifticons(&gifticons)?;
                Ok(ToolResult::success(format!("Successfully added gifticon: {} (ID: {})", name, id)))
            }
            "list" => {
                let available: Vec<Gifticon> = gifticons.iter().filter(|g| g.status == "available").cloned().collect();
                let output = serde_json::to_string_pretty(&available)?;
                Ok(ToolResult::success(output))
            }
            "use" => {
                let id = args["gifticon_id"].as_str().context("gifticon_id is required for 'use'")?;
                let mut found = false;
                for g in &mut gifticons {
                    if g.id == id {
                        g.status = "used".to_string();
                        found = true;
                        break;
                    }
                }
                
                if found {
                    self.save_gifticons(&gifticons)?;
                    Ok(ToolResult::success(format!("Gifticon {} marked as used.", id)))
                } else {
                    Ok(ToolResult::error(format!("Gifticon {} not found.", id)))
                }
            }
            "check_expiring" => {
                let threshold = args["days_threshold"].as_i64().unwrap_or(7);
                let now = Utc::now().naive_utc().date();
                let limit = now + Duration::days(threshold);
                
                let expiring: Vec<Gifticon> = gifticons.iter()
                    .filter(|g| g.status == "available")
                    .filter(|g| {
                        if let Ok(expiry_date) = NaiveDate::parse_from_str(&g.expiry_date, "%Y-%m-%d") {
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

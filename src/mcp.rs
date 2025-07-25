use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::env;
use tracing::{info, warn};

use crate::types::{AutomationRequest, BrowserAction, TaskPlan, TaskStep};

#[derive(Debug, Clone)]
pub enum MistralMode {
    Local,
    Cloud,
}

pub struct MCPClient {
    client: Client,
    mode: MistralMode,
    api_endpoint: String,
    local_endpoint: Option<String>,
    api_key: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct MistralRequest {
    model: String,
    messages: Vec<Message>,
    temperature: f32,
    max_tokens: Option<usize>,
    tools: Option<Vec<Tool>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct OllamaRequest {
    model: String,
    messages: Vec<Message>,
    stream: bool,
    options: Option<OllamaOptions>,
}

#[derive(Debug, Serialize, Deserialize)]
struct OllamaOptions {
    temperature: f32,
    num_predict: Option<usize>,
}

#[derive(Debug, Serialize, Deserialize)]
struct OllamaResponse {
    message: ResponseMessage,
    done: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Tool {
    #[serde(rename = "type")]
    tool_type: String,
    function: ToolFunction,
}

#[derive(Debug, Serialize, Deserialize)]
struct ToolFunction {
    name: String,
    description: String,
    parameters: Value,
}

#[derive(Debug, Serialize, Deserialize)]
struct MistralResponse {
    choices: Vec<Choice>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Choice {
    message: ResponseMessage,
}

#[derive(Debug, Serialize, Deserialize)]
struct ResponseMessage {
    content: Option<String>,
    tool_calls: Option<Vec<ToolCall>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ToolCall {
    function: ToolCallFunction,
}

#[derive(Debug, Serialize, Deserialize)]
struct ToolCallFunction {
    name: String,
    arguments: String,
}

impl MCPClient {
    pub async fn new() -> anyhow::Result<Self> {
        let mode = match env::var("MISTRAL_MODE").as_deref() {
            Ok("local") => MistralMode::Local,
            _ => MistralMode::Cloud,
        };

        let api_endpoint = env::var("MISTRAL_API_ENDPOINT")
            .unwrap_or_else(|_| "https://api.mistral.ai/v1/chat/completions".to_string());

        let local_endpoint = env::var("MISTRAL_LOCAL_ENDPOINT").ok();
        let api_key = env::var("MISTRAL_API_KEY").ok();

        match mode {
            MistralMode::Local => {
                if local_endpoint.is_none() {
                    warn!("MISTRAL_LOCAL_ENDPOINT not set for local mode. Using default: http://localhost:11434");
                }
                info!("Using local Mistral service via Ollama");
            }
            MistralMode::Cloud => {
                if api_key.is_none() {
                    warn!("MISTRAL_API_KEY not set. LLM features will be limited.");
                }
                info!("Using cloud Mistral API");
            }
        }

        Ok(Self {
            client: Client::new(),
            mode,
            api_endpoint,
            local_endpoint,
            api_key,
        })
    }

    pub async fn process_automation_request(
        &self,
        request: &AutomationRequest,
    ) -> anyhow::Result<TaskPlan> {
        info!(
            "Processing automation request: {}",
            request.task_description
        );

        match self.mode {
            MistralMode::Local => self.process_with_local_ollama(request).await,
            MistralMode::Cloud => self.process_with_cloud_api(request).await,
        }
    }

    async fn process_with_local_ollama(
        &self,
        request: &AutomationRequest,
    ) -> anyhow::Result<TaskPlan> {
        let default_endpoint = "http://localhost:11434".to_string();
        let endpoint = self.local_endpoint.as_ref().unwrap_or(&default_endpoint);

        let chat_endpoint = format!("{}/api/chat", endpoint);

        let system_prompt = self.get_system_prompt();
        let user_prompt = self.format_user_prompt_for_ollama(request);

        let ollama_request = OllamaRequest {
            model: "mistral:latest".to_string(),
            messages: vec![
                Message {
                    role: "system".to_string(),
                    content: system_prompt,
                },
                Message {
                    role: "user".to_string(),
                    content: user_prompt,
                },
            ],
            stream: false,
            options: Some(OllamaOptions {
                temperature: 0.1,
                num_predict: Some(2000),
            }),
        };

        let response = self
            .client
            .post(&chat_endpoint)
            .json(&ollama_request)
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to send request to local Ollama: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            warn!(
                "Local Ollama error {}: {}, falling back to simple plan",
                status, error_text
            );
            return Ok(self.create_fallback_plan(request));
        }

        let ollama_response: OllamaResponse = response
            .json()
            .await
            .map_err(|e| {
                warn!(
                    "Failed to parse Ollama response: {}, falling back to simple plan",
                    e
                );
                e
            })
            .unwrap_or_else(|_| {
                // If parsing fails, create a mock response to trigger fallback
                OllamaResponse {
                    message: ResponseMessage {
                        content: None,
                        tool_calls: None,
                    },
                    done: true,
                }
            });

        self.parse_ollama_task_plan(&ollama_response, request)
    }

    async fn process_with_cloud_api(
        &self,
        request: &AutomationRequest,
    ) -> anyhow::Result<TaskPlan> {
        // If no API key is available, return a simple fallback plan
        if self.api_key.is_none() {
            return Ok(self.create_fallback_plan(request));
        }

        let tools = self.get_browser_tools();
        let system_prompt = self.get_system_prompt();
        let user_prompt = self.format_user_prompt(request);

        let mistral_request = MistralRequest {
            model: "mistral-large-latest".to_string(),
            messages: vec![
                Message {
                    role: "system".to_string(),
                    content: system_prompt,
                },
                Message {
                    role: "user".to_string(),
                    content: user_prompt,
                },
            ],
            temperature: 0.1,
            max_tokens: Some(2000),
            tools: Some(tools),
        };

        let mut request_builder = self.client.post(&self.api_endpoint).json(&mistral_request);

        if let Some(api_key) = &self.api_key {
            request_builder = request_builder.bearer_auth(api_key);
        }

        let response = request_builder
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to send request to Mistral: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!(
                "Mistral API error {}: {}",
                status,
                error_text
            ));
        }

        let mistral_response: MistralResponse = response
            .json()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to parse Mistral response: {}", e))?;

        self.parse_task_plan(&mistral_response, request)
    }

    fn get_system_prompt(&self) -> String {
        "You are a web automation assistant. Your job is to create detailed task plans for browser automation.

Given a user's automation request, you should break it down into specific browser actions using the available tools.

Available browser actions:
- click: Click on an element using CSS selector
- type: Type text into an input field
- wait: Wait for a specified duration
- waitForElement: Wait for an element to appear
- scroll: Scroll the page in a direction
- screenshot: Take a screenshot
- getPageSource: Get the HTML source of the page
- executeScript: Execute custom JavaScript

Always provide step-by-step instructions with clear CSS selectors and expected outcomes.
Be specific about selectors - prefer IDs and classes over generic tags.
Include wait steps when necessary to ensure page elements are loaded.

Return your plan as a JSON object.".to_string()
    }

    fn format_user_prompt(&self, request: &AutomationRequest) -> String {
        let mut prompt = format!("Task: {}", request.task_description);

        if let Some(url) = &request.target_url {
            prompt.push_str(&format!("\nTarget URL: {url}"));
        }

        if let Some(context) = &request.context {
            prompt.push_str(&format!("\nAdditional context: {context:?}"));
        }

        prompt.push_str("\n\nPlease create a detailed task plan for this automation request.");
        prompt
    }

    fn format_user_prompt_for_ollama(&self, request: &AutomationRequest) -> String {
        let mut prompt = format!("Task: {}", request.task_description);

        if let Some(url) = &request.target_url {
            prompt.push_str(&format!("\nTarget URL: {url}"));
        }

        if let Some(context) = &request.context {
            prompt.push_str(&format!("\nAdditional context: {context:?}"));
        }

        prompt.push_str("\n\nPlease create a detailed task plan for this automation request. ");
        prompt.push_str("Return your response as a JSON object with the following structure:\n");
        prompt.push_str("{\n");
        prompt.push_str("  \"description\": \"Overall task description\",\n");
        prompt.push_str("  \"steps\": [\n");
        prompt.push_str("    {\n");
        prompt.push_str("      \"id\": \"unique_step_id\",\n");
        prompt.push_str("      \"action\": {\"Click\": {\"selector\": \"css_selector\"}},\n");
        prompt.push_str("      \"description\": \"What this step does\",\n");
        prompt.push_str("      \"expected_outcome\": \"What should happen\"\n");
        prompt.push_str("    }\n");
        prompt.push_str("  ]\n");
        prompt.push_str("}\n\n");
        prompt.push_str("Available actions: Click, Type, Wait, WaitForElement, Scroll, Screenshot, GetPageSource, ExecuteScript");

        prompt
    }

    fn get_browser_tools(&self) -> Vec<Tool> {
        vec![Tool {
            tool_type: "function".to_string(),
            function: ToolFunction {
                name: "create_task_plan".to_string(),
                description: "Create a browser automation task plan".to_string(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "description": {
                            "type": "string",
                            "description": "Overall description of the task"
                        },
                        "steps": {
                            "type": "array",
                            "items": {
                                "type": "object",
                                "properties": {
                                    "id": {"type": "string"},
                                    "action": {"type": "object"},
                                    "description": {"type": "string"},
                                    "expected_outcome": {"type": "string"}
                                }
                            }
                        }
                    },
                    "required": ["description", "steps"]
                }),
            },
        }]
    }

    fn parse_task_plan(
        &self,
        response: &MistralResponse,
        request: &AutomationRequest,
    ) -> anyhow::Result<TaskPlan> {
        if let Some(choice) = response.choices.first() {
            // Try to parse tool calls first
            if let Some(tool_calls) = &choice.message.tool_calls {
                for tool_call in tool_calls {
                    if tool_call.function.name == "create_task_plan" {
                        let plan: TaskPlan = serde_json::from_str(&tool_call.function.arguments)
                            .map_err(|e| anyhow::anyhow!("Failed to parse task plan: {}", e))?;
                        return Ok(plan);
                    }
                }
            }

            // Fallback to parsing content
            if let Some(content) = &choice.message.content {
                // Try to extract JSON from the content
                if let Some(start) = content.find('{') {
                    if let Some(end) = content.rfind('}') {
                        let json_str = &content[start..=end];
                        if let Ok(plan) = serde_json::from_str::<TaskPlan>(json_str) {
                            return Ok(plan);
                        }
                    }
                }
            }
        }

        // If parsing fails, return a fallback plan
        Ok(self.create_fallback_plan(request))
    }

    fn parse_ollama_task_plan(
        &self,
        response: &OllamaResponse,
        request: &AutomationRequest,
    ) -> anyhow::Result<TaskPlan> {
        if let Some(content) = &response.message.content {
            // Try to extract JSON from the content
            if let Some(start) = content.find('{') {
                if let Some(end) = content.rfind('}') {
                    let json_str = &content[start..=end];
                    if let Ok(plan) = serde_json::from_str::<TaskPlan>(json_str) {
                        info!("Successfully parsed task plan from Ollama response");
                        return Ok(plan);
                    } else {
                        warn!("Failed to parse JSON from Ollama response: {}", json_str);
                    }
                }
            }
        }

        // If parsing fails, return a fallback plan
        warn!("Could not parse task plan from Ollama, using fallback");
        Ok(self.create_fallback_plan(request))
    }

    fn create_fallback_plan(&self, request: &AutomationRequest) -> TaskPlan {
        let mut steps = Vec::new();

        // If target URL is provided, start with navigation
        if let Some(url) = &request.target_url {
            steps.push(TaskStep {
                id: "navigate".to_string(),
                action: BrowserAction::ExecuteScript {
                    script: format!("window.location.href = '{url}'"),
                },
                description: format!("Navigate to {url}"),
                expected_outcome: Some("Page should load".to_string()),
            });

            steps.push(TaskStep {
                id: "wait_load".to_string(),
                action: BrowserAction::Wait { duration_ms: 3000 },
                description: "Wait for page to load".to_string(),
                expected_outcome: Some("Page elements should be available".to_string()),
            });
        }

        // Add a generic screenshot step
        steps.push(TaskStep {
            id: "screenshot".to_string(),
            action: BrowserAction::Screenshot,
            description: "Take a screenshot for reference".to_string(),
            expected_outcome: Some("Screenshot captured".to_string()),
        });

        TaskPlan {
            description: format!("Fallback plan for: {}", request.task_description),
            steps,
        }
    }
}

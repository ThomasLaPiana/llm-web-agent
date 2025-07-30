use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::env;
use tracing::{info, warn};

use crate::types::{AutomationRequest, BrowserAction, ProductInfo, TaskPlan, TaskStep};

#[derive(Debug, Clone)]
pub enum LlamaMode {
    Local, // Using local Ollama
           // Could add cloud options later if needed
}

pub struct LlamaClient {
    client: Client,
    mode: LlamaMode,
    ollama_endpoint: String,
    mcp_endpoint: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct OllamaRequest {
    model: String,
    messages: Vec<Message>,
    stream: bool,
    tools: Option<Vec<Tool>>,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Message {
    role: String,
    content: String,
    tool_calls: Option<Vec<ToolCall>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Tool {
    #[serde(rename = "type")]
    tool_type: String,
    function: ToolFunction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ToolFunction {
    name: String,
    description: String,
    parameters: Value,
}

#[derive(Debug, Serialize, Deserialize)]
struct ResponseMessage {
    content: Option<String>,
    tool_calls: Option<Vec<ToolCall>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ToolCall {
    id: Option<String>,
    #[serde(rename = "type")]
    call_type: Option<String>,
    function: ToolCallFunction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ToolCallFunction {
    name: String,
    arguments: String,
}

impl LlamaClient {
    pub async fn new() -> anyhow::Result<Self> {
        let ollama_endpoint =
            env::var("OLLAMA_ENDPOINT").unwrap_or_else(|_| "http://localhost:11434".to_string());

        let mcp_endpoint =
            env::var("MCP_ENDPOINT").unwrap_or_else(|_| "http://localhost:3000".to_string());

        info!("Using Ollama endpoint: {}", ollama_endpoint);
        info!("Using MCP endpoint: {}", mcp_endpoint);

        Ok(Self {
            client: Client::new(),
            mode: LlamaMode::Local,
            ollama_endpoint,
            mcp_endpoint,
        })
    }

    pub async fn extract_product_information(
        &self,
        url: &str,
        html_content: &str,
    ) -> anyhow::Result<ProductInfo> {
        info!("Extracting product information using Llama + MCP tools");
        info!("URL: {}", url);
        info!("HTML content length: {} characters", html_content.len());

        // First, let's get the available MCP tools
        let tools = self.get_mcp_tools().await?;

        // Create a conversation with the Llama model
        let system_prompt = self.get_enhanced_product_extraction_prompt();
        let user_prompt = format!(
            "I need to extract product information from this web page. The URL is: {}\n\n\
            I have the raw HTML content available. Please use the appropriate tools to:\n\
            1. First analyze the page structure to understand what kind of site this is\n\
            2. Extract clean, structured product data\n\
            3. Return the product information in a clear format\n\n\
            Start by analyzing the page structure.",
            url
        );

        let mut messages = vec![
            Message {
                role: "system".to_string(),
                content: system_prompt,
                tool_calls: None,
            },
            Message {
                role: "user".to_string(),
                content: user_prompt,
                tool_calls: None,
            },
        ];

        // Run the conversation with tool calling
        let mut conversation_turns = 0;
        let max_turns = 5;

        while conversation_turns < max_turns {
            let response = self.call_llama_with_tools(&messages, &tools).await?;

            if let Some(tool_calls) = &response.message.tool_calls {
                // Execute tool calls
                for tool_call in tool_calls {
                    let tool_result = self.execute_mcp_tool(tool_call, html_content, url).await?;

                    // Add tool result to conversation
                    messages.push(Message {
                        role: "assistant".to_string(),
                        content: response.message.content.clone().unwrap_or_default(),
                        tool_calls: Some(vec![tool_call.clone()]),
                    });

                    messages.push(Message {
                        role: "tool".to_string(),
                        content: tool_result,
                        tool_calls: None,
                    });
                }

                conversation_turns += 1;
            } else {
                // No more tool calls, parse final response
                if let Some(content) = &response.message.content {
                    return self.parse_final_product_response(content);
                }
                break;
            }
        }

        // Fallback if we couldn't get a good response
        warn!("Could not extract product information using MCP tools, using fallback");
        Ok(self.create_fallback_product_info())
    }

    async fn get_mcp_tools(&self) -> anyhow::Result<Vec<Tool>> {
        let manifest_url = format!("{}/.well-known/mcp/manifest.json", self.mcp_endpoint);

        let response = self
            .client
            .get(&manifest_url)
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to fetch MCP manifest: {}", e))?;

        let manifest: Value = response
            .json()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to parse MCP manifest: {}", e))?;

        let mut tools = Vec::new();

        if let Some(tool_list) = manifest.get("tools").and_then(|t| t.as_array()) {
            for tool_def in tool_list {
                if let (Some(name), Some(description), Some(input_schema)) = (
                    tool_def.get("name").and_then(|n| n.as_str()),
                    tool_def.get("description").and_then(|d| d.as_str()),
                    tool_def.get("input_schema"),
                ) {
                    tools.push(Tool {
                        tool_type: "function".to_string(),
                        function: ToolFunction {
                            name: name.to_string(),
                            description: description.to_string(),
                            parameters: input_schema.clone(),
                        },
                    });
                }
            }
        }

        info!("Loaded {} MCP tools", tools.len());
        Ok(tools)
    }

    async fn call_llama_with_tools(
        &self,
        messages: &[Message],
        tools: &[Tool],
    ) -> anyhow::Result<OllamaResponse> {
        let chat_endpoint = format!("{}/api/chat", self.ollama_endpoint);

        // Use a capable Llama model with function calling support
        let model = env::var("LLAMA_MODEL").unwrap_or_else(|_| "llama3.2:latest".to_string());

        let request = OllamaRequest {
            model,
            messages: messages.to_vec(),
            stream: false,
            tools: if tools.is_empty() {
                None
            } else {
                Some(tools.to_vec())
            },
            options: Some(OllamaOptions {
                temperature: 0.1,
                num_predict: Some(2000),
            }),
        };

        info!("Calling Llama model with {} tools available", tools.len());

        let response = self
            .client
            .post(&chat_endpoint)
            .json(&request)
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to call Ollama: {}", e))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!("Ollama API error: {}", error_text));
        }

        let ollama_response: OllamaResponse = response
            .json()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to parse Ollama response: {}", e))?;

        Ok(ollama_response)
    }

    async fn execute_mcp_tool(
        &self,
        tool_call: &ToolCall,
        html_content: &str,
        url: &str,
    ) -> anyhow::Result<String> {
        let mcp_url = format!("{}/mcp", self.mcp_endpoint);

        // Parse the tool arguments
        let arguments: Value =
            serde_json::from_str(&tool_call.function.arguments).unwrap_or_else(|_| json!({}));

        // Add HTML content to arguments if not present
        let mut final_arguments = arguments;
        if final_arguments.get("html_content").is_none() {
            final_arguments["html_content"] = json!(html_content);
        }
        if final_arguments.get("url").is_none() {
            final_arguments["url"] = json!(url);
        }

        let mcp_request = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/call",
            "params": {
                "name": tool_call.function.name,
                "arguments": final_arguments
            }
        });

        info!("Executing MCP tool: {}", tool_call.function.name);

        let response = self
            .client
            .post(&mcp_url)
            .json(&mcp_request)
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to call MCP tool: {}", e))?;

        let mcp_response: Value = response
            .json()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to parse MCP response: {}", e))?;

        if let Some(error) = mcp_response.get("error") {
            return Err(anyhow::anyhow!("MCP tool error: {}", error));
        }

        if let Some(result) = mcp_response.get("result") {
            Ok(serde_json::to_string_pretty(result).unwrap_or_default())
        } else {
            Err(anyhow::anyhow!("No result from MCP tool"))
        }
    }

    fn get_enhanced_product_extraction_prompt(&self) -> String {
        "You are an expert web scraping assistant with access to specialized HTML parsing tools. \
        Your job is to extract product information from e-commerce websites using the available tools.

Available tools:
- analyze_page_structure: Identifies the type of e-commerce platform and suggests extraction strategies
- extract_product_data: Uses CSS selectors and JSON-LD to extract structured product information
- extract_clean_text: Removes clutter and extracts clean, readable content
- extract_by_selectors: Extract specific data using custom CSS selectors

Best practices:
1. Always start by analyzing the page structure to understand the website type
2. Use extract_product_data for comprehensive product extraction
3. If extract_product_data doesn't work well, use extract_by_selectors with specific selectors
4. Focus on extracting: name, price, description, availability, brand, rating, image URL
5. Return results in a clear, structured format

Work step by step and use the most appropriate tools for each task.".to_string()
    }

    fn parse_final_product_response(&self, content: &str) -> anyhow::Result<ProductInfo> {
        info!("Parsing final product response: {}", content);

        // Try to extract JSON from the response
        if let Some(start) = content.find('{') {
            if let Some(end) = content.rfind('}') {
                let json_str = &content[start..=end];
                if let Ok(parsed) = serde_json::from_str::<Value>(json_str) {
                    return Ok(ProductInfo {
                        name: parsed
                            .get("name")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string()),
                        description: parsed
                            .get("description")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string()),
                        price: parsed
                            .get("price")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string()),
                        availability: parsed
                            .get("availability")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string()),
                        brand: parsed
                            .get("brand")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string()),
                        rating: parsed
                            .get("rating")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string()),
                        image_url: parsed
                            .get("image_url")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string()),
                        raw_data: Some(content.to_string()),
                        raw_llm_response: Some(content.to_string()),
                    });
                }
            }
        }

        // Fallback: try to parse structured text response
        Ok(self.parse_text_response(content))
    }

    fn parse_text_response(&self, content: &str) -> ProductInfo {
        let mut product_info = ProductInfo {
            name: None,
            description: None,
            price: None,
            availability: None,
            brand: None,
            rating: None,
            image_url: None,
            raw_data: Some(content.to_string()),
            raw_llm_response: Some(content.to_string()),
        };

        // Simple text parsing for common patterns
        let lines: Vec<&str> = content.lines().collect();
        for line in lines {
            let line = line.trim();
            if line.to_lowercase().contains("name:") || line.to_lowercase().contains("product:") {
                if let Some(name) = line.split(':').nth(1) {
                    product_info.name = Some(name.trim().to_string());
                }
            } else if line.to_lowercase().contains("price:") {
                if let Some(price) = line.split(':').nth(1) {
                    product_info.price = Some(price.trim().to_string());
                }
            } else if line.to_lowercase().contains("brand:") {
                if let Some(brand) = line.split(':').nth(1) {
                    product_info.brand = Some(brand.trim().to_string());
                }
            } else if line.to_lowercase().contains("description:") {
                if let Some(description) = line.split(':').nth(1) {
                    product_info.description = Some(description.trim().to_string());
                }
            }
        }

        product_info
    }

    fn create_fallback_product_info(&self) -> ProductInfo {
        ProductInfo {
            name: Some("Unable to extract product name with MCP tools".to_string()),
            description: Some(
                "Product information extraction failed using Llama + MCP".to_string(),
            ),
            price: None,
            availability: None,
            brand: None,
            rating: None,
            image_url: None,
            raw_data: None,
            raw_llm_response: Some("Fallback mode - MCP extraction failed".to_string()),
        }
    }

    // Automation functionality (you can extend this later)
    pub async fn process_automation_request(
        &self,
        request: &AutomationRequest,
    ) -> anyhow::Result<TaskPlan> {
        info!(
            "Processing automation request with Llama: {}",
            request.task_description
        );

        // For now, return a simple plan - you can enhance this with MCP tools later
        Ok(TaskPlan {
            description: format!("Llama-generated plan for: {}", request.task_description),
            steps: vec![TaskStep {
                id: "analyze".to_string(),
                action: BrowserAction::Screenshot,
                description: "Take screenshot to analyze page".to_string(),
                expected_outcome: Some("Screenshot captured for analysis".to_string()),
            }],
        })
    }
}

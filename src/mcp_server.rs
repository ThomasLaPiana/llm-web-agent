use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::Arc;
use tracing::info;

// MCP Protocol Structures
#[derive(Debug, Serialize, Deserialize)]
pub struct MCPRequest {
    pub jsonrpc: String,
    pub id: Option<Value>,
    pub method: String,
    pub params: Option<Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MCPResponse {
    pub jsonrpc: String,
    pub id: Option<Value>,
    pub result: Option<Value>,
    pub error: Option<MCPError>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MCPError {
    pub code: i32,
    pub message: String,
    pub data: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolInfo {
    pub name: String,
    pub description: String,
    pub input_schema: Value,
}

#[derive(Debug, Clone)]
pub struct MCPServerState {
    pub tools: Vec<ToolInfo>,
}

impl MCPServerState {
    pub fn new() -> Self {
        Self {
            tools: vec![
                ToolInfo {
                    name: "extract_clean_text".to_string(),
                    description: "Extract clean, readable text content from HTML".to_string(),
                    input_schema: json!({
                        "type": "object",
                        "properties": {
                            "html_content": {
                                "type": "string",
                                "description": "Raw HTML content to clean"
                            }
                        },
                        "required": ["html_content"]
                    }),
                },
                ToolInfo {
                    name: "extract_product_data".to_string(),
                    description: "Extract structured product information using CSS selectors"
                        .to_string(),
                    input_schema: json!({
                        "type": "object",
                        "properties": {
                            "html_content": {
                                "type": "string",
                                "description": "HTML content to parse"
                            },
                            "url": {
                                "type": "string",
                                "description": "Source URL for context"
                            }
                        },
                        "required": ["html_content"]
                    }),
                },
                ToolInfo {
                    name: "extract_by_selectors".to_string(),
                    description: "Extract specific content using CSS selectors".to_string(),
                    input_schema: json!({
                        "type": "object",
                        "properties": {
                            "html_content": {
                                "type": "string",
                                "description": "HTML content to parse"
                            },
                            "selectors": {
                                "type": "object",
                                "description": "CSS selectors to extract data",
                                "additionalProperties": {"type": "string"}
                            }
                        },
                        "required": ["html_content", "selectors"]
                    }),
                },
                ToolInfo {
                    name: "analyze_page_structure".to_string(),
                    description: "Analyze HTML structure and suggest extraction strategies"
                        .to_string(),
                    input_schema: json!({
                        "type": "object",
                        "properties": {
                            "html_content": {
                                "type": "string",
                                "description": "HTML content to analyze"
                            }
                        },
                        "required": ["html_content"]
                    }),
                },
            ],
        }
    }
}

pub fn create_mcp_router() -> Router<Arc<MCPServerState>> {
    Router::new()
        .route("/mcp", post(handle_mcp_request))
        .route("/.well-known/mcp/manifest.json", get(get_manifest))
        .with_state(Arc::new(MCPServerState::new()))
}

async fn get_manifest(State(state): State<Arc<MCPServerState>>) -> Json<Value> {
    Json(json!({
        "name": "web-content-extractor",
        "version": "1.0.0",
        "description": "Specialized tools for web content extraction and parsing",
        "tools": state.tools.iter().map(|tool| json!({
            "name": tool.name,
            "description": tool.description,
            "input_schema": tool.input_schema
        })).collect::<Vec<_>>()
    }))
}

async fn handle_mcp_request(
    State(state): State<Arc<MCPServerState>>,
    Json(request): Json<MCPRequest>,
) -> Result<Json<MCPResponse>, StatusCode> {
    info!("Received MCP request: {:?}", request);

    let response = match request.method.as_str() {
        "initialize" => handle_initialize(&request),
        "tools/list" => handle_tools_list(&state, &request),
        "tools/call" => handle_tool_call(&request).await,
        _ => MCPResponse {
            jsonrpc: "2.0".to_string(),
            id: request.id,
            result: None,
            error: Some(MCPError {
                code: -32601,
                message: "Method not found".to_string(),
                data: None,
            }),
        },
    };

    Ok(Json(response))
}

fn handle_initialize(request: &MCPRequest) -> MCPResponse {
    MCPResponse {
        jsonrpc: "2.0".to_string(),
        id: request.id.clone(),
        result: Some(json!({
            "capabilities": {
                "tools": true,
                "resources": false,
                "prompts": false
            },
            "serverInfo": {
                "name": "web-content-extractor",
                "version": "1.0.0"
            }
        })),
        error: None,
    }
}

fn handle_tools_list(state: &Arc<MCPServerState>, request: &MCPRequest) -> MCPResponse {
    MCPResponse {
        jsonrpc: "2.0".to_string(),
        id: request.id.clone(),
        result: Some(json!({
            "tools": state.tools.iter().map(|tool| json!({
                "name": tool.name,
                "description": tool.description,
                "inputSchema": tool.input_schema
            })).collect::<Vec<_>>()
        })),
        error: None,
    }
}

async fn handle_tool_call(request: &MCPRequest) -> MCPResponse {
    let params = match &request.params {
        Some(params) => params,
        None => {
            return MCPResponse {
                jsonrpc: "2.0".to_string(),
                id: request.id.clone(),
                result: None,
                error: Some(MCPError {
                    code: -32602,
                    message: "Invalid params".to_string(),
                    data: None,
                }),
            };
        }
    };

    let tool_name = params["name"].as_str().unwrap_or("");
    let arguments = &params["arguments"];

    let result = match tool_name {
        "extract_clean_text" => extract_clean_text(arguments).await,
        "extract_product_data" => extract_product_data(arguments).await,
        "extract_by_selectors" => extract_by_selectors(arguments).await,
        "analyze_page_structure" => analyze_page_structure(arguments).await,
        _ => Err(format!("Unknown tool: {}", tool_name)),
    };

    match result {
        Ok(content) => MCPResponse {
            jsonrpc: "2.0".to_string(),
            id: request.id.clone(),
            result: Some(content),
            error: None,
        },
        Err(error) => MCPResponse {
            jsonrpc: "2.0".to_string(),
            id: request.id.clone(),
            result: None,
            error: Some(MCPError {
                code: -32603,
                message: error,
                data: None,
            }),
        },
    }
}

// Tool implementations
async fn extract_clean_text(arguments: &Value) -> Result<Value, String> {
    let html_content = arguments["html_content"]
        .as_str()
        .ok_or("Missing html_content parameter")?;

    let document = Html::parse_document(html_content);

    // Remove script and style elements
    let _script_selector = Selector::parse("script, style, nav, header, footer, aside").unwrap();
    let _clean_html = html_content.to_string();

    // Extract main content
    let main_selectors = [
        "main",
        "article",
        "[role='main']",
        ".main-content",
        "#main-content",
        ".content",
        "#content",
    ];

    let mut extracted_text = String::new();

    for selector_str in &main_selectors {
        if let Ok(selector) = Selector::parse(selector_str) {
            for element in document.select(&selector) {
                let text = element.text().collect::<Vec<_>>().join(" ");
                if !text.trim().is_empty() {
                    extracted_text = text;
                    break;
                }
            }
            if !extracted_text.is_empty() {
                break;
            }
        }
    }

    // Fallback to body text if no main content found
    if extracted_text.is_empty() {
        if let Ok(body_selector) = Selector::parse("body") {
            for element in document.select(&body_selector) {
                extracted_text = element.text().collect::<Vec<_>>().join(" ");
                break;
            }
        }
    }

    // Clean up whitespace
    let cleaned = extracted_text
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");

    Ok(json!({
        "clean_text": cleaned,
        "length": cleaned.len(),
        "extraction_method": "semantic_selectors"
    }))
}

async fn extract_product_data(arguments: &Value) -> Result<Value, String> {
    let html_content = arguments["html_content"]
        .as_str()
        .ok_or("Missing html_content parameter")?;
    let url = arguments["url"].as_str().unwrap_or("");

    let document = Html::parse_document(html_content);
    let mut product_data = json!({});

    // Common product selectors for different e-commerce sites
    let product_selectors = [
        // Amazon
        (
            "name",
            vec!["#productTitle", "h1.a-size-large", ".product-title"],
        ),
        (
            "price",
            vec![
                "[data-testid='price']",
                ".a-price-whole",
                ".price",
                ".current-price",
                "[data-price]",
            ],
        ),
        (
            "description",
            vec![
                "[data-feature-name='productDescription']",
                ".product-description",
                "#description",
            ],
        ),
        (
            "availability",
            vec!["#availability span", ".availability", "#stock-status"],
        ),
        ("brand", vec!["[data-testid='brand']", ".brand", "#brand"]),
        (
            "rating",
            vec![
                "[data-testid='rating']",
                ".a-icon-alt",
                ".rating",
                ".star-rating",
            ],
        ),
        (
            "image",
            vec![
                "[data-testid='image']",
                "#landingImage",
                ".product-image img",
                ".main-image img",
            ],
        ),
    ];

    for (field, selectors) in &product_selectors {
        for selector_str in selectors {
            if let Ok(selector) = Selector::parse(selector_str) {
                for element in document.select(&selector) {
                    let value = if *field == "image" {
                        element.value().attr("src").unwrap_or("").to_string()
                    } else {
                        element
                            .text()
                            .collect::<Vec<_>>()
                            .join(" ")
                            .trim()
                            .to_string()
                    };

                    if !value.is_empty() {
                        product_data[field] = json!(value);
                        break;
                    }
                }
                if product_data[field] != json!(null) {
                    break;
                }
            }
        }
    }

    // Try JSON-LD structured data
    if let Ok(script_selector) = Selector::parse("script[type='application/ld+json']") {
        for element in document.select(&script_selector) {
            let script_content = element.text().collect::<String>();
            if let Ok(json_ld) = serde_json::from_str::<Value>(&script_content) {
                if let Some(product_json) = extract_product_from_jsonld(&json_ld) {
                    // Merge JSON-LD data, preferring existing data
                    for (key, value) in product_json.as_object().unwrap() {
                        if product_data[key] == json!(null) {
                            product_data[key] = value.clone();
                        }
                    }
                }
            }
        }
    }

    Ok(json!({
        "url": url,
        "extracted_data": product_data,
        "extraction_timestamp": chrono::Utc::now().to_rfc3339(),
        "extraction_method": "css_selectors_and_jsonld"
    }))
}

async fn extract_by_selectors(arguments: &Value) -> Result<Value, String> {
    let html_content = arguments["html_content"]
        .as_str()
        .ok_or("Missing html_content parameter")?;
    let selectors = arguments["selectors"]
        .as_object()
        .ok_or("Missing selectors parameter")?;

    let document = Html::parse_document(html_content);
    let mut results = json!({});

    for (key, selector_str) in selectors {
        let selector_str = selector_str.as_str().unwrap_or("");

        if let Ok(selector) = Selector::parse(selector_str) {
            let mut values = Vec::new();

            for element in document.select(&selector) {
                let text = element
                    .text()
                    .collect::<Vec<_>>()
                    .join(" ")
                    .trim()
                    .to_string();
                if !text.is_empty() {
                    values.push(text);
                }
            }

            results[key] = if values.len() == 1 {
                json!(values[0])
            } else {
                json!(values)
            };
        }
    }

    Ok(results)
}

async fn analyze_page_structure(arguments: &Value) -> Result<Value, String> {
    let html_content = arguments["html_content"]
        .as_str()
        .ok_or("Missing html_content parameter")?;

    let document = Html::parse_document(html_content);

    // Analyze common e-commerce patterns
    let mut analysis = json!({
        "detected_patterns": [],
        "suggested_selectors": {},
        "content_sections": []
    });

    // Check for common e-commerce indicators
    let ecommerce_indicators = [
        ("amazon", vec![".a-price", "#productTitle", "#acrPopover"]),
        ("shopify", vec![".product-form", ".price", ".product-title"]),
        (
            "woocommerce",
            vec![".woocommerce", ".price", ".product_title"],
        ),
        (
            "magento",
            vec![
                ".product-info-price",
                ".product-title",
                ".product-info-main",
            ],
        ),
    ];

    for (platform, indicators) in &ecommerce_indicators {
        let mut matches = 0;
        for indicator in indicators {
            if let Ok(selector) = Selector::parse(indicator) {
                if document.select(&selector).next().is_some() {
                    matches += 1;
                }
            }
        }
        if matches > 0 {
            analysis["detected_patterns"]
                .as_array_mut()
                .unwrap()
                .push(json!({
                    "platform": platform,
                    "confidence": (matches as f32 / indicators.len() as f32) * 100.0
                }));
        }
    }

    Ok(analysis)
}

fn extract_product_from_jsonld(json_ld: &Value) -> Option<Value> {
    // Handle both single objects and arrays
    let items = if json_ld.is_array() {
        json_ld.as_array()?
    } else {
        std::slice::from_ref(json_ld)
    };

    for item in items {
        if let Some(type_val) = item.get("@type") {
            if type_val == "Product" {
                let mut product = json!({});

                if let Some(name) = item.get("name") {
                    product["name"] = name.clone();
                }
                if let Some(description) = item.get("description") {
                    product["description"] = description.clone();
                }
                if let Some(brand) = item.get("brand") {
                    product["brand"] = if brand.is_string() {
                        brand.clone()
                    } else if let Some(brand_name) = brand.get("name") {
                        brand_name.clone()
                    } else {
                        json!(null)
                    };
                }
                if let Some(offers) = item.get("offers") {
                    if let Some(price) = offers.get("price") {
                        product["price"] = price.clone();
                    }
                    if let Some(availability) = offers.get("availability") {
                        product["availability"] = availability.clone();
                    }
                }
                if let Some(aggregate_rating) = item.get("aggregateRating") {
                    if let Some(rating_value) = aggregate_rating.get("ratingValue") {
                        product["rating"] = rating_value.clone();
                    }
                }
                if let Some(image) = item.get("image") {
                    product["image_url"] = if image.is_string() {
                        image.clone()
                    } else if image.is_array() && !image.as_array().unwrap().is_empty() {
                        image.as_array().unwrap()[0].clone()
                    } else {
                        json!(null)
                    };
                }

                return Some(product);
            }
        }
    }

    None
}

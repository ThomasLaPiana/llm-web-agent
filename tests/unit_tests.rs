use llm_web_agent::types::*;
use serde_json::json;
use std::collections::HashMap;

// Test data constants
const TEST_SELECTOR: &str = "#submit-button";
const TEST_EMAIL_SELECTOR: &str = "#email";
const TEST_EMAIL: &str = "test@example.com";
const TEST_SESSION_ID: &str = "test-session";
const TEST_URL: &str = "https://example.com";
const TEST_SCRIPT: &str = "document.title";
const TEST_WAIT_DURATION: u64 = 1000;
const TEST_SCROLL_PIXELS: i32 = 200;

// === Test Data Builders ===

/// Builder for creating test browser actions
pub struct BrowserActionBuilder;

impl BrowserActionBuilder {
    pub fn click(selector: &str) -> BrowserAction {
        BrowserAction::Click {
            selector: selector.to_string(),
        }
    }

    pub fn type_text(selector: &str, text: &str) -> BrowserAction {
        BrowserAction::Type {
            selector: selector.to_string(),
            text: text.to_string(),
        }
    }

    pub fn wait(duration_ms: u64) -> BrowserAction {
        BrowserAction::Wait { duration_ms }
    }

    pub fn scroll(direction: ScrollDirection, pixels: Option<i32>) -> BrowserAction {
        BrowserAction::Scroll { direction, pixels }
    }

    pub fn screenshot() -> BrowserAction {
        BrowserAction::Screenshot
    }

    pub fn execute_script(script: &str) -> BrowserAction {
        BrowserAction::ExecuteScript {
            script: script.to_string(),
        }
    }
}

/// Builder for creating test requests
pub struct RequestBuilder;

impl RequestBuilder {
    pub fn navigation(session_id: &str, url: &str) -> NavigateRequest {
        NavigateRequest {
            session_id: session_id.to_string(),
            url: url.to_string(),
        }
    }

    pub fn interaction(session_id: &str, action: BrowserAction) -> InteractionRequest {
        InteractionRequest {
            session_id: session_id.to_string(),
            action,
        }
    }

    pub fn automation(
        session_id: &str,
        task_description: &str,
        target_url: Option<&str>,
        context: Option<HashMap<String, serde_json::Value>>,
    ) -> AutomationRequest {
        AutomationRequest {
            session_id: session_id.to_string(),
            task_description: task_description.to_string(),
            target_url: target_url.map(|s| s.to_string()),
            context,
        }
    }
}

/// Helper function to test action serialization round-trip
fn test_action_serialization(action: BrowserAction) -> BrowserAction {
    let serialized = serde_json::to_string(&action).expect("Should serialize");
    serde_json::from_str(&serialized).expect("Should deserialize")
}

// === Browser Action Serialization Tests ===

#[test]
fn test_click_action_serialization() {
    let action = BrowserActionBuilder::click(TEST_SELECTOR);
    let deserialized = test_action_serialization(action);

    match deserialized {
        BrowserAction::Click { selector } => {
            assert_eq!(selector, TEST_SELECTOR);
        }
        _ => panic!("Wrong action type"),
    }
}

#[test]
fn test_type_action_serialization() {
    let action = BrowserActionBuilder::type_text(TEST_EMAIL_SELECTOR, TEST_EMAIL);
    let deserialized = test_action_serialization(action);

    match deserialized {
        BrowserAction::Type { selector, text } => {
            assert_eq!(selector, TEST_EMAIL_SELECTOR);
            assert_eq!(text, TEST_EMAIL);
        }
        _ => panic!("Wrong action type"),
    }
}

#[test]
fn test_wait_action_serialization() {
    let action = BrowserActionBuilder::wait(TEST_WAIT_DURATION);
    let deserialized = test_action_serialization(action);

    match deserialized {
        BrowserAction::Wait { duration_ms } => {
            assert_eq!(duration_ms, TEST_WAIT_DURATION);
        }
        _ => panic!("Wrong action type"),
    }
}

#[test]
fn test_scroll_action_serialization() {
    let action = BrowserActionBuilder::scroll(ScrollDirection::Down, Some(TEST_SCROLL_PIXELS));
    let deserialized = test_action_serialization(action);

    match deserialized {
        BrowserAction::Scroll { direction, pixels } => {
            assert!(matches!(direction, ScrollDirection::Down));
            assert_eq!(pixels, Some(TEST_SCROLL_PIXELS));
        }
        _ => panic!("Wrong action type"),
    }
}

#[test]
fn test_screenshot_action_serialization() {
    let action = BrowserActionBuilder::screenshot();
    let deserialized = test_action_serialization(action);

    assert!(matches!(deserialized, BrowserAction::Screenshot));
}

#[test]
fn test_execute_script_action_serialization() {
    let action = BrowserActionBuilder::execute_script(TEST_SCRIPT);
    let deserialized = test_action_serialization(action);

    match deserialized {
        BrowserAction::ExecuteScript { script } => {
            assert_eq!(script, TEST_SCRIPT);
        }
        _ => panic!("Wrong action type"),
    }
}

#[test]
fn test_scroll_direction_variants() {
    let directions = vec![
        ScrollDirection::Up,
        ScrollDirection::Down,
        ScrollDirection::Left,
        ScrollDirection::Right,
    ];

    for direction in directions {
        let action = BrowserActionBuilder::scroll(direction.clone(), Some(100));
        let deserialized = test_action_serialization(action);

        match deserialized {
            BrowserAction::Scroll {
                direction: d,
                pixels,
            } => {
                assert_eq!(format!("{d:?}"), format!("{:?}", direction));
                assert_eq!(pixels, Some(100));
            }
            _ => panic!("Wrong action type"),
        }
    }
}

// === Request Serialization Tests ===

#[test]
fn test_navigation_request_serialization() {
    let nav_request = RequestBuilder::navigation(TEST_SESSION_ID, TEST_URL);

    let serialized = serde_json::to_string(&nav_request).expect("Should serialize");
    let expected = json!({
        "session_id": TEST_SESSION_ID,
        "url": TEST_URL
    });

    let deserialized: serde_json::Value =
        serde_json::from_str(&serialized).expect("Should deserialize");
    assert_eq!(deserialized, expected);
}

#[test]
fn test_interaction_request_serialization() {
    let action = BrowserActionBuilder::click("#button");
    let interaction_request = RequestBuilder::interaction(TEST_SESSION_ID, action);

    let serialized = serde_json::to_string(&interaction_request).expect("Should serialize");
    let deserialized: InteractionRequest =
        serde_json::from_str(&serialized).expect("Should deserialize");

    assert_eq!(deserialized.session_id, TEST_SESSION_ID);
    match deserialized.action {
        BrowserAction::Click { selector } => {
            assert_eq!(selector, "#button");
        }
        _ => panic!("Wrong action type"),
    }
}

#[test]
fn test_automation_request_serialization() {
    let context = Some(HashMap::from([
        ("email".to_string(), json!(TEST_EMAIL)),
        ("name".to_string(), json!("Test User")),
    ]));

    let auto_request = RequestBuilder::automation(
        TEST_SESSION_ID,
        "Fill out form",
        Some(TEST_URL),
        context,
    );

    let serialized = serde_json::to_string(&auto_request).expect("Should serialize");
    let deserialized: AutomationRequest =
        serde_json::from_str(&serialized).expect("Should deserialize");

    assert_eq!(deserialized.session_id, TEST_SESSION_ID);
    assert_eq!(deserialized.task_description, "Fill out form");
    assert_eq!(deserialized.target_url, Some(TEST_URL.to_string()));
    assert!(deserialized.context.is_some());

    let context = deserialized.context.unwrap();
    assert_eq!(context.get("email").unwrap(), &json!(TEST_EMAIL));
    assert_eq!(context.get("name").unwrap(), &json!("Test User"));
}

// === Task Plan and Result Tests ===

#[test]
fn test_task_plan_creation() {
    let task_plan = TaskPlan {
        description: "Test automation task".to_string(),
        steps: vec![
            TaskStep {
                id: "step1".to_string(),
                action: BrowserActionBuilder::click("#submit"),
                description: "Click submit button".to_string(),
                expected_outcome: Some("Form submitted".to_string()),
            },
            TaskStep {
                id: "step2".to_string(),
                action: BrowserActionBuilder::wait(TEST_WAIT_DURATION),
                description: "Wait for response".to_string(),
                expected_outcome: Some("Page loads".to_string()),
            },
        ],
    };

    assert_eq!(task_plan.description, "Test automation task");
    assert_eq!(task_plan.steps.len(), 2);
    assert_eq!(task_plan.steps[0].id, "step1");
    assert_eq!(task_plan.steps[1].id, "step2");

    // Test that actions are correctly constructed
    match &task_plan.steps[0].action {
        BrowserAction::Click { selector } => {
            assert_eq!(selector, "#submit");
        }
        _ => panic!("Wrong action type for step1"),
    }

    match &task_plan.steps[1].action {
        BrowserAction::Wait { duration_ms } => {
            assert_eq!(*duration_ms, TEST_WAIT_DURATION);
        }
        _ => panic!("Wrong action type for step2"),
    }
}

#[test]
fn test_task_result_success() {
    let task_result = TaskResult {
        step_id: "step1".to_string(),
        success: true,
        output: Some("Button clicked successfully".to_string()),
        error: None,
    };

    assert_eq!(task_result.step_id, "step1");
    assert!(task_result.success);
    assert!(task_result.output.is_some());
    assert!(task_result.error.is_none());
    assert_eq!(
        task_result.output.unwrap(),
        "Button clicked successfully"
    );
}

#[test]
fn test_task_result_failure() {
    let failed_result = TaskResult {
        step_id: "step2".to_string(),
        success: false,
        output: None,
        error: Some("Element not found".to_string()),
    };

    assert_eq!(failed_result.step_id, "step2");
    assert!(!failed_result.success);
    assert!(failed_result.output.is_none());
    assert!(failed_result.error.is_some());
    assert_eq!(failed_result.error.unwrap(), "Element not found");
}

// === Error Type Tests ===

#[test]
fn test_app_error_browser_error() {
    let browser_error = AppError::BrowserError("Failed to click".to_string());
    let error_string = browser_error.to_string();
    
    assert!(error_string.contains("Browser error"));
    assert!(error_string.contains("Failed to click"));
}

#[test]
fn test_app_error_session_not_found() {
    let session_error = AppError::SessionNotFound("session-123".to_string());
    let error_string = session_error.to_string();
    
    assert!(error_string.contains("Session not found"));
    assert!(error_string.contains("session-123"));
}

#[test]
fn test_app_error_mcp_error() {
    let mcp_error = AppError::MCPError("API key invalid".to_string());
    let error_string = mcp_error.to_string();
    
    assert!(error_string.contains("MCP error"));
    assert!(error_string.contains("API key invalid"));
}

use llm_web_agent::types::*;
use serde_json::json;

#[test]
fn test_browser_action_serialization() {
    // Test Click action
    let click_action = BrowserAction::Click {
        selector: "#submit-button".to_string(),
    };

    let serialized = serde_json::to_string(&click_action).expect("Should serialize");
    let deserialized: BrowserAction =
        serde_json::from_str(&serialized).expect("Should deserialize");

    match deserialized {
        BrowserAction::Click { selector } => {
            assert_eq!(selector, "#submit-button");
        }
        _ => panic!("Wrong action type"),
    }
}

#[test]
fn test_type_action_serialization() {
    let type_action = BrowserAction::Type {
        selector: "#email".to_string(),
        text: "test@example.com".to_string(),
    };

    let serialized = serde_json::to_string(&type_action).expect("Should serialize");
    let deserialized: BrowserAction =
        serde_json::from_str(&serialized).expect("Should deserialize");

    match deserialized {
        BrowserAction::Type { selector, text } => {
            assert_eq!(selector, "#email");
            assert_eq!(text, "test@example.com");
        }
        _ => panic!("Wrong action type"),
    }
}

#[test]
fn test_wait_action_serialization() {
    let wait_action = BrowserAction::Wait { duration_ms: 1000 };

    let serialized = serde_json::to_string(&wait_action).expect("Should serialize");
    let deserialized: BrowserAction =
        serde_json::from_str(&serialized).expect("Should deserialize");

    match deserialized {
        BrowserAction::Wait { duration_ms } => {
            assert_eq!(duration_ms, 1000);
        }
        _ => panic!("Wrong action type"),
    }
}

#[test]
fn test_scroll_action_serialization() {
    let scroll_action = BrowserAction::Scroll {
        direction: ScrollDirection::Down,
        pixels: Some(200),
    };

    let serialized = serde_json::to_string(&scroll_action).expect("Should serialize");
    let deserialized: BrowserAction =
        serde_json::from_str(&serialized).expect("Should deserialize");

    match deserialized {
        BrowserAction::Scroll { direction, pixels } => {
            assert!(matches!(direction, ScrollDirection::Down));
            assert_eq!(pixels, Some(200));
        }
        _ => panic!("Wrong action type"),
    }
}

#[test]
fn test_screenshot_action_serialization() {
    let screenshot_action = BrowserAction::Screenshot;

    let serialized = serde_json::to_string(&screenshot_action).expect("Should serialize");
    let deserialized: BrowserAction =
        serde_json::from_str(&serialized).expect("Should deserialize");

    assert!(matches!(deserialized, BrowserAction::Screenshot));
}

#[test]
fn test_execute_script_action_serialization() {
    let script_action = BrowserAction::ExecuteScript {
        script: "document.title".to_string(),
    };

    let serialized = serde_json::to_string(&script_action).expect("Should serialize");
    let deserialized: BrowserAction =
        serde_json::from_str(&serialized).expect("Should deserialize");

    match deserialized {
        BrowserAction::ExecuteScript { script } => {
            assert_eq!(script, "document.title");
        }
        _ => panic!("Wrong action type"),
    }
}

#[test]
fn test_navigation_request_serialization() {
    let nav_request = NavigateRequest {
        session_id: "test-session".to_string(),
        url: "https://example.com".to_string(),
    };

    let serialized = serde_json::to_string(&nav_request).expect("Should serialize");
    let expected = json!({
        "session_id": "test-session",
        "url": "https://example.com"
    });

    let deserialized: serde_json::Value =
        serde_json::from_str(&serialized).expect("Should deserialize");
    assert_eq!(deserialized, expected);
}

#[test]
fn test_interaction_request_serialization() {
    let interaction_request = InteractionRequest {
        session_id: "test-session".to_string(),
        action: BrowserAction::Click {
            selector: "#button".to_string(),
        },
    };

    let serialized = serde_json::to_string(&interaction_request).expect("Should serialize");
    let deserialized: InteractionRequest =
        serde_json::from_str(&serialized).expect("Should deserialize");

    assert_eq!(deserialized.session_id, "test-session");
    match deserialized.action {
        BrowserAction::Click { selector } => {
            assert_eq!(selector, "#button");
        }
        _ => panic!("Wrong action type"),
    }
}

#[test]
fn test_automation_request_serialization() {
    let auto_request = AutomationRequest {
        session_id: "test-session".to_string(),
        task_description: "Fill out form".to_string(),
        target_url: Some("https://example.com".to_string()),
        context: Some(std::collections::HashMap::from([
            ("email".to_string(), json!("test@example.com")),
            ("name".to_string(), json!("Test User")),
        ])),
    };

    let serialized = serde_json::to_string(&auto_request).expect("Should serialize");
    let deserialized: AutomationRequest =
        serde_json::from_str(&serialized).expect("Should deserialize");

    assert_eq!(deserialized.session_id, "test-session");
    assert_eq!(deserialized.task_description, "Fill out form");
    assert_eq!(
        deserialized.target_url,
        Some("https://example.com".to_string())
    );
    assert!(deserialized.context.is_some());
}

#[test]
fn test_task_plan_creation() {
    let task_plan = TaskPlan {
        description: "Test automation task".to_string(),
        steps: vec![
            TaskStep {
                id: "step1".to_string(),
                action: BrowserAction::Click {
                    selector: "#submit".to_string(),
                },
                description: "Click submit button".to_string(),
                expected_outcome: Some("Form submitted".to_string()),
            },
            TaskStep {
                id: "step2".to_string(),
                action: BrowserAction::Wait { duration_ms: 1000 },
                description: "Wait for response".to_string(),
                expected_outcome: Some("Page loads".to_string()),
            },
        ],
    };

    assert_eq!(task_plan.description, "Test automation task");
    assert_eq!(task_plan.steps.len(), 2);
    assert_eq!(task_plan.steps[0].id, "step1");
    assert_eq!(task_plan.steps[1].id, "step2");
}

#[test]
fn test_task_result_creation() {
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
        let action = BrowserAction::Scroll {
            direction: direction.clone(),
            pixels: Some(100),
        };

        let serialized = serde_json::to_string(&action).expect("Should serialize");
        let deserialized: BrowserAction =
            serde_json::from_str(&serialized).expect("Should deserialize");

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

#[test]
fn test_app_error_display() {
    let browser_error = AppError::BrowserError("Failed to click".to_string());
    assert!(browser_error.to_string().contains("Browser error"));
    assert!(browser_error.to_string().contains("Failed to click"));

    let session_error = AppError::SessionNotFound("session-123".to_string());
    assert!(session_error.to_string().contains("Session not found"));
    assert!(session_error.to_string().contains("session-123"));

    let mcp_error = AppError::MCPError("API key invalid".to_string());
    assert!(mcp_error.to_string().contains("MCP error"));
    assert!(mcp_error.to_string().contains("API key invalid"));
}

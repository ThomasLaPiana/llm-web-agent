use anyhow::{anyhow, Result};
use chromiumoxide::browser::{Browser, BrowserConfig};
use chromiumoxide::page::Page;
use futures::StreamExt;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::OnceCell;
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::types::{BrowserAction, ScrollDirection, TaskPlan, TaskResult};

// Global browser singleton
static BROWSER_SINGLETON: OnceCell<Arc<Browser>> = OnceCell::const_new();

// Initialize the global browser instance
async fn get_or_create_browser() -> Result<Arc<Browser>> {
    BROWSER_SINGLETON
        .get_or_try_init(|| async {
            info!("Creating browser singleton instance");

            let (browser, mut handler) = Browser::launch(
                BrowserConfig::builder()
                    .args(vec![
                        "--headless",
                        "--no-sandbox",
                        "--disable-dev-shm-usage",
                        "--disable-gpu",
                        "--remote-debugging-port=0",
                    ])
                    .build()
                    .map_err(|e| anyhow!("Failed to build browser config: {}", e))?,
            )
            .await
            .map_err(|e| anyhow!("Failed to launch browser: {}", e))?;

            // Spawn task to handle browser events
            tokio::task::spawn(async move {
                while let Some(h) = handler.next().await {
                    if h.is_err() {
                        error!("Browser handler error: {:?}", h);
                        break;
                    }
                }
            });

            info!("Browser singleton created successfully");
            Ok(Arc::new(browser))
        })
        .await
        .map(|browser| browser.clone())
}

#[allow(dead_code)]
pub struct BrowserSession {
    browser: Arc<Browser>,
    page: Page,
    session_id: String,
}

impl BrowserSession {
    pub async fn new() -> Result<Self> {
        info!("Creating new browser session");

        // Get the shared browser instance
        let browser = get_or_create_browser().await?;

        // Create a new page in the existing browser with retry logic
        let page = match browser.new_page("about:blank").await {
            Ok(page) => {
                info!("Successfully created new page in browser");
                page
            }
            Err(e) => {
                warn!("First attempt to create page failed: {}, retrying...", e);

                // Wait a moment and retry
                tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;

                browser
                    .new_page("about:blank")
                    .await
                    .map_err(|e| anyhow!("Failed to create new page after retry: {}", e))?
            }
        };

        let session_id = Uuid::new_v4().to_string();
        info!(
            "Browser session created successfully with ID: {}",
            session_id
        );

        Ok(Self {
            browser,
            page,
            session_id,
        })
    }

    pub async fn navigate(&mut self, url: &str) -> Result<()> {
        info!("Navigating to: {}", url);

        // Simple navigation without waiting for navigation events
        // This avoids WebSocket communication issues with wait_for_navigation
        let navigation_result = tokio::time::timeout(tokio::time::Duration::from_secs(30), async {
            self.page
                .goto(url)
                .await
                .map_err(|e| anyhow!("Failed to navigate to {}: {}", url, e))?;

            // Give the page a moment to start loading, but don't wait for navigation events
            tokio::time::sleep(tokio::time::Duration::from_millis(2000)).await;

            Ok(())
        })
        .await
        .map_err(|_| anyhow!("Navigation timeout after 30 seconds"))?;

        navigation_result
    }

    pub async fn interact(&mut self, action: &BrowserAction) -> Result<String> {
        match action {
            BrowserAction::Click { selector } => {
                info!("Clicking element: {}", selector);
                let element = self
                    .page
                    .find_element(selector)
                    .await
                    .map_err(|e| anyhow!("Element not found {}: {}", selector, e))?;

                element
                    .click()
                    .await
                    .map_err(|e| anyhow!("Failed to click element: {}", e))?;

                Ok("Click successful".to_string())
            }

            BrowserAction::Type { selector, text } => {
                info!("Typing '{}' into element: {}", text, selector);
                let element = self
                    .page
                    .find_element(selector)
                    .await
                    .map_err(|e| anyhow!("Element not found {}: {}", selector, e))?;

                element
                    .click()
                    .await
                    .map_err(|e| anyhow!("Failed to focus element: {}", e))?;

                element
                    .type_str(text)
                    .await
                    .map_err(|e| anyhow!("Failed to type text: {}", e))?;

                Ok("Text input successful".to_string())
            }

            BrowserAction::Wait { duration_ms } => {
                info!("Waiting for {} ms", duration_ms);
                tokio::time::sleep(tokio::time::Duration::from_millis(*duration_ms)).await;
                Ok("Wait completed".to_string())
            }

            BrowserAction::WaitForElement {
                selector,
                timeout_ms,
            } => {
                info!("Waiting for element: {}", selector);
                let timeout = timeout_ms.unwrap_or(30000);

                // Wait using a loop with timeout
                let start = std::time::Instant::now();
                loop {
                    if self.page.find_element(selector).await.is_ok() {
                        break;
                    }
                    if start.elapsed().as_millis() > timeout as u128 {
                        return Err(anyhow!("Element not found within {}ms", timeout));
                    }
                    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                }

                Ok("Element found".to_string())
            }

            BrowserAction::Scroll { direction, pixels } => {
                let pixels = pixels.unwrap_or(100);
                let (x, y) = match direction {
                    ScrollDirection::Up => (0, -pixels),
                    ScrollDirection::Down => (0, pixels),
                    ScrollDirection::Left => (-pixels, 0),
                    ScrollDirection::Right => (pixels, 0),
                };

                info!("Scrolling by ({}, {})", x, y);
                let scroll_script = format!("window.scrollBy({x}, {y})");
                self.page
                    .evaluate(scroll_script.as_str())
                    .await
                    .map_err(|e| anyhow!("Failed to scroll: {}", e))?;

                Ok(format!("Scrolled by ({x}, {y})"))
            }

            BrowserAction::Screenshot => {
                info!("Taking screenshot");
                let screenshot = self
                    .page
                    .screenshot(chromiumoxide::page::ScreenshotParams::builder().build())
                    .await
                    .map_err(|e| anyhow!("Failed to take screenshot: {}", e))?;

                // Convert to base64 for response
                use base64::Engine;
                let base64_image = base64::engine::general_purpose::STANDARD.encode(&screenshot);
                Ok(format!("data:image/png;base64,{base64_image}"))
            }

            BrowserAction::GetPageSource => {
                info!("Getting page source");
                let source = self
                    .page
                    .content()
                    .await
                    .map_err(|e| anyhow!("Failed to get page source: {}", e))?;

                Ok(source)
            }

            BrowserAction::ExecuteScript { script } => {
                info!("Executing script: {}", script);
                let result = self
                    .page
                    .evaluate(script.as_str())
                    .await
                    .map_err(|e| anyhow!("Failed to execute script: {}", e))?;

                Ok(format!("{:?}", result.value()))
            }
        }
    }

    pub async fn extract_data(&self, selector: &str) -> Result<HashMap<String, Value>> {
        info!("Extracting data using selector: {}", selector);

        let script = format!(
            r#"
            Array.from(document.querySelectorAll('{selector}')).map(el => {{
                return {{
                    text: el.textContent || el.innerText || '',
                    html: el.innerHTML,
                    attributes: Object.fromEntries(
                        Array.from(el.attributes).map(attr => [attr.name, attr.value])
                    ),
                    tagName: el.tagName.toLowerCase(),
                    className: el.className,
                    id: el.id
                }};
            }})
            "#
        );

        let result = self
            .page
            .evaluate(script.as_str())
            .await
            .map_err(|e| anyhow!("Failed to extract data: {}", e))?;

        let mut data = HashMap::new();
        if let Some(value) = result.value() {
            data.insert("elements".to_string(), value.clone());
        }
        data.insert("count".to_string(), serde_json::json!(0)); // TODO: Calculate count

        Ok(data)
    }

    pub async fn execute_task_plan(&mut self, plan: &TaskPlan) -> Result<Vec<TaskResult>> {
        info!("Executing task plan: {}", plan.description);
        let mut results = Vec::new();

        for step in &plan.steps {
            info!("Executing step: {}", step.description);

            match self.interact(&step.action).await {
                Ok(output) => {
                    results.push(TaskResult {
                        step_id: step.id.clone(),
                        success: true,
                        output: Some(output),
                        error: None,
                    });
                }
                Err(e) => {
                    let error_msg = e.to_string();
                    error!("Step failed: {}", error_msg);

                    results.push(TaskResult {
                        step_id: step.id.clone(),
                        success: false,
                        output: None,
                        error: Some(error_msg),
                    });

                    // Continue execution even if a step fails
                    warn!("Continuing execution despite step failure");
                }
            }

            // Small delay between steps
            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        }

        Ok(results)
    }

    pub async fn get_current_url(&self) -> Result<String> {
        let url = self
            .page
            .url()
            .await
            .map_err(|e| anyhow!("Failed to get current URL: {}", e))?;

        Ok(url.unwrap_or_else(|| "about:blank".to_string()))
    }
}

impl Drop for BrowserSession {
    fn drop(&mut self) {
        info!("Dropping browser session");
        // The browser will be closed when dropped
    }
}

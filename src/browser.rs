use chromiumoxide::{Browser, BrowserConfig, Page};
use futures::StreamExt;
use serde_json::Value;
use std::collections::HashMap;
use tracing::{error, info, warn};

use crate::types::{BrowserAction, ScrollDirection, TaskPlan, TaskResult};

pub struct BrowserSession {
    #[allow(dead_code)]
    browser: Browser,
    page: Page,
}

impl BrowserSession {
    pub async fn new() -> anyhow::Result<Self> {
        info!("Creating new browser session");

        let (browser, mut handler) = Browser::launch(
            BrowserConfig::builder()
                .with_head()
                .args(vec![
                    "--no-sandbox",
                    "--disable-setuid-sandbox",
                    "--disable-dev-shm-usage",
                    "--disable-accelerated-2d-canvas",
                    "--no-first-run",
                    "--no-zygote",
                    "--disable-gpu",
                ])
                .build()
                .map_err(|e| anyhow::anyhow!("Failed to build browser config: {}", e))?,
        )
        .await
        .map_err(|e| anyhow::anyhow!("Failed to launch browser: {}", e))?;

        // Spawn task to handle browser events
        tokio::task::spawn(async move {
            while let Some(h) = handler.next().await {
                if h.is_err() {
                    error!("Browser handler error: {:?}", h);
                    break;
                }
            }
        });

        let page = browser
            .new_page("about:blank")
            .await
            .map_err(|e| anyhow::anyhow!("Failed to create new page: {}", e))?;

        Ok(Self { browser, page })
    }

    pub async fn navigate(&mut self, url: &str) -> anyhow::Result<()> {
        info!("Navigating to: {}", url);

        self.page
            .goto(url)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to navigate to {}: {}", url, e))?;

        // Wait for page load
        self.page
            .wait_for_navigation()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to wait for navigation: {}", e))?;

        Ok(())
    }

    pub async fn interact(&mut self, action: &BrowserAction) -> anyhow::Result<String> {
        match action {
            BrowserAction::Click { selector } => {
                info!("Clicking element: {}", selector);
                let element = self
                    .page
                    .find_element(selector)
                    .await
                    .map_err(|e| anyhow::anyhow!("Element not found {}: {}", selector, e))?;

                element
                    .click()
                    .await
                    .map_err(|e| anyhow::anyhow!("Failed to click element: {}", e))?;

                Ok("Click successful".to_string())
            }

            BrowserAction::Type { selector, text } => {
                info!("Typing '{}' into element: {}", text, selector);
                let element = self
                    .page
                    .find_element(selector)
                    .await
                    .map_err(|e| anyhow::anyhow!("Element not found {}: {}", selector, e))?;

                element
                    .click()
                    .await
                    .map_err(|e| anyhow::anyhow!("Failed to focus element: {}", e))?;

                element
                    .type_str(text)
                    .await
                    .map_err(|e| anyhow::anyhow!("Failed to type text: {}", e))?;

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
                        return Err(anyhow::anyhow!("Element not found within {}ms", timeout));
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
                    .map_err(|e| anyhow::anyhow!("Failed to scroll: {}", e))?;

                Ok(format!("Scrolled by ({x}, {y})"))
            }

            BrowserAction::Screenshot => {
                info!("Taking screenshot");
                let screenshot = self
                    .page
                    .screenshot(chromiumoxide::page::ScreenshotParams::builder().build())
                    .await
                    .map_err(|e| anyhow::anyhow!("Failed to take screenshot: {}", e))?;

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
                    .map_err(|e| anyhow::anyhow!("Failed to get page source: {}", e))?;

                Ok(source)
            }

            BrowserAction::ExecuteScript { script } => {
                info!("Executing script: {}", script);
                let result = self
                    .page
                    .evaluate(script.as_str())
                    .await
                    .map_err(|e| anyhow::anyhow!("Failed to execute script: {}", e))?;

                Ok(format!("{:?}", result.value()))
            }
        }
    }

    pub async fn extract_data(&self, selector: &str) -> anyhow::Result<HashMap<String, Value>> {
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
            .map_err(|e| anyhow::anyhow!("Failed to extract data: {}", e))?;

        let mut data = HashMap::new();
        if let Some(value) = result.value() {
            data.insert("elements".to_string(), value.clone());
        }
        data.insert("count".to_string(), serde_json::json!(0)); // TODO: Calculate count

        Ok(data)
    }

    pub async fn execute_task_plan(&mut self, plan: &TaskPlan) -> anyhow::Result<Vec<TaskResult>> {
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

    pub async fn get_current_url(&self) -> anyhow::Result<String> {
        let url = self
            .page
            .url()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to get current URL: {}", e))?;

        Ok(url.unwrap_or_else(|| "about:blank".to_string()))
    }
}

impl Drop for BrowserSession {
    fn drop(&mut self) {
        info!("Dropping browser session");
        // The browser will be closed when dropped
    }
}

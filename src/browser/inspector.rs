//! DOM element inspector and interaction tools

use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use tracing::{debug, info, error};
use anyhow::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElementSelector {
    /// CSS selector
    pub css: Option<String>,
    /// XPath selector
    pub xpath: Option<String>,
    /// Element text content
    pub text: Option<String>,
    /// Element attributes to match
    pub attributes: Option<HashMap<String, String>>,
    /// Element index if multiple matches
    pub index: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElementInfo {
    /// Element tag name
    pub tag_name: String,
    /// Element attributes
    pub attributes: HashMap<String, String>,
    /// Element text content
    pub text_content: String,
    /// Element inner HTML
    pub inner_html: String,
    /// Element outer HTML
    pub outer_html: String,
    /// Computed styles
    pub computed_styles: HashMap<String, String>,
    /// Element dimensions and position
    pub bounds: ElementBounds,
    /// Whether element is visible
    pub is_visible: bool,
    /// Whether element is enabled
    pub is_enabled: bool,
    /// CSS selector path to element
    pub css_path: String,
    /// XPath to element
    pub xpath: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElementBounds {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    /// Coordinates relative to viewport
    pub viewport_x: f64,
    pub viewport_y: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InteractionType {
    Click,
    DoubleClick,
    RightClick,
    Hover,
    Focus,
    Blur,
    Type { text: String },
    Clear,
    Select { value: String },
    Check,
    Uncheck,
    Upload { file_path: String },
    Scroll { x: i32, y: i32 },
    Drag { to_x: f64, to_y: f64 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InteractionResult {
    pub success: bool,
    pub error: Option<String>,
    pub element_info: Option<ElementInfo>,
    pub screenshot_data: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WaitCondition {
    pub condition_type: WaitConditionType,
    pub selector: ElementSelector,
    pub timeout_ms: u32,
    pub poll_interval_ms: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WaitConditionType {
    ElementVisible,
    ElementHidden,
    ElementEnabled,
    ElementDisabled,
    ElementTextContains { text: String },
    ElementAttributeEquals { attribute: String, value: String },
    ElementCount { count: usize },
    PageTitleContains { text: String },
    UrlContains { text: String },
}

pub struct DOMInspector {
    /// JavaScript snippets for DOM operations
    js_snippets: HashMap<String, String>,
}

impl DOMInspector {
    pub fn new() -> Self {
        let mut inspector = Self {
            js_snippets: HashMap::new(),
        };
        inspector.initialize_js_snippets();
        inspector
    }

    fn initialize_js_snippets(&mut self) {
        // Element selection JavaScript
        self.js_snippets.insert("find_element".to_string(), r#"
function findElement(selector) {
    let element = null;
    
    if (selector.css) {
        const elements = document.querySelectorAll(selector.css);
        element = selector.index !== undefined ? elements[selector.index] : elements[0];
    } else if (selector.xpath) {
        const result = document.evaluate(selector.xpath, document, null, XPathResult.FIRST_ORDERED_NODE_TYPE, null);
        element = result.singleNodeValue;
    } else if (selector.text) {
        const walker = document.createTreeWalker(document.body, NodeFilter.SHOW_TEXT);
        let node;
        while (node = walker.nextNode()) {
            if (node.textContent.includes(selector.text)) {
                element = node.parentElement;
                break;
            }
        }
    }
    
    return element;
}
        "#.to_string());

        // Element information extraction
        self.js_snippets.insert("get_element_info".to_string(), r#"
function getElementInfo(element) {
    if (!element) return null;
    
    const rect = element.getBoundingClientRect();
    const computedStyle = window.getComputedStyle(element);
    const attributes = {};
    
    for (let attr of element.attributes) {
        attributes[attr.name] = attr.value;
    }
    
    const styles = {};
    for (let prop of ['display', 'visibility', 'opacity', 'color', 'background-color', 'font-size', 'font-family']) {
        styles[prop] = computedStyle.getPropertyValue(prop);
    }
    
    // Generate CSS selector path
    function getCSSPath(el) {
        if (!(el instanceof Element)) return;
        const path = [];
        while (el.nodeType === Node.ELEMENT_NODE) {
            let selector = el.nodeName.toLowerCase();
            if (el.id) {
                selector += '#' + el.id;
                path.unshift(selector);
                break;
            } else {
                let sib = el, nth = 1;
                while (sib = sib.previousElementSibling) {
                    if (sib.nodeName.toLowerCase() == selector) nth++;
                }
                if (nth != 1) selector += ":nth-of-type(" + nth + ")";
            }
            path.unshift(selector);
            el = el.parentNode;
        }
        return path.join(" > ");
    }
    
    // Generate XPath
    function getXPath(el) {
        if (el.id !== '') return 'id("' + el.id + '")';
        if (el === document.body) return el.tagName;
        
        let ix = 0;
        const siblings = el.parentNode.childNodes;
        for (let i = 0; i < siblings.length; i++) {
            const sibling = siblings[i];
            if (sibling === el) return getXPath(el.parentNode) + '/' + el.tagName + '[' + (ix + 1) + ']';
            if (sibling.nodeType === 1 && sibling.tagName === el.tagName) ix++;
        }
    }
    
    return {
        tagName: element.tagName.toLowerCase(),
        attributes: attributes,
        textContent: element.textContent.trim(),
        innerHTML: element.innerHTML,
        outerHTML: element.outerHTML,
        computedStyles: styles,
        bounds: {
            x: rect.left + window.scrollX,
            y: rect.top + window.scrollY,
            width: rect.width,
            height: rect.height,
            viewport_x: rect.left,
            viewport_y: rect.top
        },
        isVisible: rect.width > 0 && rect.height > 0 && computedStyle.visibility !== 'hidden' && computedStyle.display !== 'none',
        isEnabled: !element.disabled,
        cssPath: getCSSPath(element),
        xpath: getXPath(element)
    };
}
        "#.to_string());

        // Element interaction JavaScript
        self.js_snippets.insert("interact_element".to_string(), r#"
function interactWithElement(element, interaction) {
    if (!element) return { success: false, error: 'Element not found' };
    
    try {
        switch (interaction.type) {
            case 'click':
                element.click();
                break;
            case 'double_click':
                element.dispatchEvent(new MouseEvent('dblclick', { bubbles: true }));
                break;
            case 'right_click':
                element.dispatchEvent(new MouseEvent('contextmenu', { bubbles: true }));
                break;
            case 'hover':
                element.dispatchEvent(new MouseEvent('mouseover', { bubbles: true }));
                break;
            case 'focus':
                element.focus();
                break;
            case 'blur':
                element.blur();
                break;
            case 'type':
                element.focus();
                element.value = interaction.text;
                element.dispatchEvent(new Event('input', { bubbles: true }));
                element.dispatchEvent(new Event('change', { bubbles: true }));
                break;
            case 'clear':
                element.value = '';
                element.dispatchEvent(new Event('input', { bubbles: true }));
                element.dispatchEvent(new Event('change', { bubbles: true }));
                break;
            case 'select':
                if (element.tagName.toLowerCase() === 'select') {
                    element.value = interaction.value;
                    element.dispatchEvent(new Event('change', { bubbles: true }));
                }
                break;
            case 'check':
                if (element.type === 'checkbox' || element.type === 'radio') {
                    element.checked = true;
                    element.dispatchEvent(new Event('change', { bubbles: true }));
                }
                break;
            case 'uncheck':
                if (element.type === 'checkbox') {
                    element.checked = false;
                    element.dispatchEvent(new Event('change', { bubbles: true }));
                }
                break;
            case 'scroll':
                element.scrollBy(interaction.x, interaction.y);
                break;
            default:
                return { success: false, error: 'Unknown interaction type: ' + interaction.type };
        }
        
        return { success: true };
    } catch (error) {
        return { success: false, error: error.message };
    }
}
        "#.to_string());

        // Wait condition checking
        self.js_snippets.insert("check_wait_condition".to_string(), r#"
function checkWaitCondition(condition) {
    const element = findElement(condition.selector);
    
    switch (condition.condition_type) {
        case 'element_visible':
            return element && getElementInfo(element).isVisible;
        case 'element_hidden':
            return !element || !getElementInfo(element).isVisible;
        case 'element_enabled':
            return element && getElementInfo(element).isEnabled;
        case 'element_disabled':
            return element && !getElementInfo(element).isEnabled;
        case 'element_text_contains':
            return element && element.textContent.includes(condition.text);
        case 'element_attribute_equals':
            return element && element.getAttribute(condition.attribute) === condition.value;
        case 'element_count':
            const elements = condition.selector.css ? document.querySelectorAll(condition.selector.css) : [];
            return elements.length === condition.count;
        case 'page_title_contains':
            return document.title.includes(condition.text);
        case 'url_contains':
            return window.location.href.includes(condition.text);
        default:
            return false;
    }
}
        "#.to_string());

        // Highlight element for debugging
        self.js_snippets.insert("highlight_element".to_string(), r#"
function highlightElement(element, color = '#ff0000', duration = 3000) {
    if (!element) return;
    
    const originalStyle = {
        outline: element.style.outline,
        outlineOffset: element.style.outlineOffset
    };
    
    element.style.outline = `3px solid ${color}`;
    element.style.outlineOffset = '2px';
    
    setTimeout(() => {
        element.style.outline = originalStyle.outline;
        element.style.outlineOffset = originalStyle.outlineOffset;
    }, duration);
}
        "#.to_string());
    }

    /// Find element using various selector strategies
    pub fn find_element(&self, selector: &ElementSelector) -> String {
        let selector_json = serde_json::to_string(selector).unwrap_or_default();
        format!(
            r#"
            {};
            const selector = {};
            const element = findElement(selector);
            element ? getElementInfo(element) : null;
            "#,
            self.js_snippets.get("find_element").unwrap(),
            selector_json
        )
    }

    /// Get detailed information about an element
    pub fn get_element_info(&self, selector: &ElementSelector) -> String {
        let selector_json = serde_json::to_string(selector).unwrap_or_default();
        format!(
            r#"
            {};
            {};
            const selector = {};
            const element = findElement(selector);
            element ? getElementInfo(element) : null;
            "#,
            self.js_snippets.get("find_element").unwrap(),
            self.js_snippets.get("get_element_info").unwrap(),
            selector_json
        )
    }

    /// Interact with an element
    pub fn interact_with_element(&self, selector: &ElementSelector, interaction: &InteractionType) -> String {
        let selector_json = serde_json::to_string(selector).unwrap_or_default();
        let interaction_json = serde_json::to_string(interaction).unwrap_or_default();
        format!(
            r#"
            {};
            {};
            {};
            const selector = {};
            const interaction = {};
            const element = findElement(selector);
            const result = interactWithElement(element, interaction);
            if (result.success) {{
                result.elementInfo = getElementInfo(element);
            }}
            result;
            "#,
            self.js_snippets.get("find_element").unwrap(),
            self.js_snippets.get("get_element_info").unwrap(),
            self.js_snippets.get("interact_element").unwrap(),
            selector_json,
            interaction_json
        )
    }

    /// Highlight an element for visual debugging
    pub fn highlight_element(&self, selector: &ElementSelector, color: Option<&str>) -> String {
        let selector_json = serde_json::to_string(selector).unwrap_or_default();
        let color = color.unwrap_or("#ff0000");
        format!(
            r#"
            {};
            {};
            const selector = {};
            const element = findElement(selector);
            if (element) {{
                highlightElement(element, '{}');
                getElementInfo(element);
            }} else {{
                null;
            }}
            "#,
            self.js_snippets.get("find_element").unwrap(),
            self.js_snippets.get("highlight_element").unwrap(),
            selector_json,
            color
        )
    }

    /// Wait for a condition to be met
    pub fn check_wait_condition(&self, condition: &WaitCondition) -> String {
        let condition_json = serde_json::to_string(condition).unwrap_or_default();
        format!(
            r#"
            {};
            {};
            {};
            const condition = {};
            checkWaitCondition(condition);
            "#,
            self.js_snippets.get("find_element").unwrap(),
            self.js_snippets.get("get_element_info").unwrap(),
            self.js_snippets.get("check_wait_condition").unwrap(),
            condition_json
        )
    }

    /// Get all elements matching a selector
    pub fn find_all_elements(&self, css_selector: &str) -> String {
        format!(
            r#"
            {};
            const elements = document.querySelectorAll('{}');
            Array.from(elements).map(el => getElementInfo(el));
            "#,
            self.js_snippets.get("get_element_info").unwrap(),
            css_selector.replace('\'', "\\'")
        )
    }

    /// Get page information
    pub fn get_page_info(&self) -> String {
        r#"
        ({
            title: document.title,
            url: window.location.href,
            readyState: document.readyState,
            elementCount: document.querySelectorAll('*').length,
            viewport: {
                width: window.innerWidth,
                height: window.innerHeight,
                scrollX: window.scrollX,
                scrollY: window.scrollY
            },
            performance: performance.now ? {
                navigationStart: performance.timing.navigationStart,
                loadEventEnd: performance.timing.loadEventEnd,
                domContentLoaded: performance.timing.domContentLoadedEventEnd - performance.timing.navigationStart,
                loadComplete: performance.timing.loadEventEnd - performance.timing.navigationStart
            } : null
        })
        "#.to_string()
    }
}

impl Default for DOMInspector {
    fn default() -> Self {
        Self::new()
    }
}

impl ElementSelector {
    pub fn css(selector: &str) -> Self {
        Self {
            css: Some(selector.to_string()),
            xpath: None,
            text: None,
            attributes: None,
            index: None,
        }
    }

    pub fn xpath(xpath: &str) -> Self {
        Self {
            css: None,
            xpath: Some(xpath.to_string()),
            text: None,
            attributes: None,
            index: None,
        }
    }

    pub fn text(text: &str) -> Self {
        Self {
            css: None,
            xpath: None,
            text: Some(text.to_string()),
            attributes: None,
            index: None,
        }
    }

    pub fn with_index(mut self, index: usize) -> Self {
        self.index = Some(index);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_element_selector_creation() {
        let css_selector = ElementSelector::css("button.submit");
        assert_eq!(css_selector.css, Some("button.submit".to_string()));
        assert!(css_selector.xpath.is_none());

        let xpath_selector = ElementSelector::xpath("//button[@class='submit']");
        assert_eq!(xpath_selector.xpath, Some("//button[@class='submit']".to_string()));
        assert!(xpath_selector.css.is_none());
    }

    #[test]
    fn test_inspector_js_generation() {
        let inspector = DOMInspector::new();
        let js = inspector.find_element(&ElementSelector::css("button"));
        assert!(js.contains("findElement"));
        assert!(js.contains("getElementInfo"));
    }
}
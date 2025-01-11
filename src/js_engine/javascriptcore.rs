use std::error::Error;
use javascriptcore_rs::{Context, Value};
use super::{JsEngine, JsEngineType};

pub struct JavaScriptCoreEngine {
    context: Context,
}

impl JavaScriptCoreEngine {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let context = Context::new();
        Ok(Self { context })
    }
}

impl JsEngine for JavaScriptCoreEngine {
    fn name(&self) -> &'static str {
        "JavaScriptCore"
    }

    fn engine_type(&self) -> JsEngineType {
        JsEngineType::JavaScriptCore
    }

    fn evaluate(&self, script: &str) -> Result<String, Box<dyn Error>> {
        let result = self.context.evaluate_script(script)?;
        Ok(result.to_string())
    }

    fn set_global(&self, name: &str, value: &str) -> Result<(), Box<dyn Error>> {
        let js_value = Value::from_str(&self.context, value);
        self.context.set_global_value(name, js_value);
        Ok(())
    }

    fn get_global(&self, name: &str) -> Result<String, Box<dyn Error>> {
        let value = self.context.get_global_value(name);
        Ok(value.to_string())
    }
} 
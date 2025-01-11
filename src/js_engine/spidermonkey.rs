use std::error::Error;
use mozjs::jsapi::JSContext;
use mozjs::jsval::JSVal;
use mozjs::rust::{Runtime, SIMPLE_GLOBAL_CLASS};
use super::{JsEngine, JsEngineType};

pub struct SpiderMonkeyEngine {
    runtime: Runtime,
    context: *mut JSContext,
}

impl SpiderMonkeyEngine {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let runtime = Runtime::new().ok_or("Failed to create runtime")?;
        let context = runtime.cx();
        
        Ok(Self {
            runtime,
            context,
        })
    }
}

impl JsEngine for SpiderMonkeyEngine {
    fn name(&self) -> &'static str {
        "SpiderMonkey"
    }

    fn engine_type(&self) -> JsEngineType {
        JsEngineType::SpiderMonkey
    }

    fn evaluate(&self, script: &str) -> Result<String, Box<dyn Error>> {
        let result = self.runtime.evaluate_script(
            SIMPLE_GLOBAL_CLASS,
            script,
            "script",
            1,
        )?;
        
        Ok(format!("{:?}", result))
    }

    fn set_global(&self, name: &str, value: &str) -> Result<(), Box<dyn Error>> {
        self.runtime.set_global_property(name, value)?;
        Ok(())
    }

    fn get_global(&self, name: &str) -> Result<String, Box<dyn Error>> {
        let value = self.runtime.get_global_property(name)?;
        Ok(format!("{:?}", value))
    }
} 
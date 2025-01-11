use std::error::Error;

pub mod v8;
pub mod javascriptcore;
pub mod spidermonkey;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum JsEngineType {
    V8,
    JavaScriptCore,
    SpiderMonkey,
}

pub trait JsEngine {
    fn name(&self) -> &'static str;
    fn engine_type(&self) -> JsEngineType;
    fn evaluate(&self, script: &str) -> Result<String, Box<dyn Error>>;
    fn set_global(&self, name: &str, value: &str) -> Result<(), Box<dyn Error>>;
    fn get_global(&self, name: &str) -> Result<String, Box<dyn Error>>;
    
    // Optional methods with default implementations
    fn init(&self) -> Result<(), Box<dyn Error>> {
        Ok(())
    }
    
    fn cleanup(&self) -> Result<(), Box<dyn Error>> {
        Ok(())
    }
    
    fn add_hook(&self, _name: &str, _callback: Box<dyn Fn(&str) -> Result<String, Box<dyn Error>>>) -> Result<(), Box<dyn Error>> {
        Ok(())
    }
}

pub struct JsEngineBuilder {
    engine_type: JsEngineType,
}

impl JsEngineBuilder {
    pub fn new(engine_type: JsEngineType) -> Self {
        Self { engine_type }
    }
    
    pub fn build(&self) -> Result<Box<dyn JsEngine>, Box<dyn Error>> {
        match self.engine_type {
            JsEngineType::V8 => Ok(Box::new(v8::V8Engine::new()?)),
            JsEngineType::JavaScriptCore => Ok(Box::new(javascriptcore::JavaScriptCoreEngine::new()?)),
            JsEngineType::SpiderMonkey => Ok(Box::new(spidermonkey::SpiderMonkeyEngine::new()?)),
        }
    }
} 
use std::error::Error;
use std::sync::Once;
use v8::{HandleScope, Context, Local, Value, Platform};
use super::{JsEngine, JsEngineType};

static V8_INIT: Once = Once::new();
static mut V8_PLATFORM: Option<Box<dyn Platform>> = None;

fn init_platform() {
    V8_INIT.call_once(|| {
        let platform = v8::new_default_platform(0, false).make_shared();
        v8::V8::initialize_platform(platform.clone());
        v8::V8::initialize();
        unsafe {
            V8_PLATFORM = Some(platform);
        }
    });
}

pub struct V8Engine {
    isolate: v8::OwnedIsolate,
    context: v8::Global<Context>,
}

impl V8Engine {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        // Initialize V8 platform if not already initialized
        init_platform();

        // Create a new isolate and context
        let mut isolate = v8::Isolate::new(Default::default());
        let handle_scope = &mut HandleScope::new(&mut isolate);
        let context = Context::new(handle_scope);
        let context = v8::Global::new(handle_scope, context);

        Ok(Self {
            isolate,
            context,
        })
    }

    fn get_context<'s>(&mut self, scope: &mut HandleScope<'s>) -> Local<'s, Context> {
        self.context.open(scope)
    }
}

impl JsEngine for V8Engine {
    fn name(&self) -> &'static str {
        "V8"
    }

    fn engine_type(&self) -> JsEngineType {
        JsEngineType::V8
    }

    fn evaluate(&self, script: &str) -> Result<String, Box<dyn Error>> {
        let mut scope = HandleScope::new(&mut self.isolate.clone());
        let context = self.context.open(&mut scope);
        let scope = &mut HandleScope::with_context(&mut scope, context);

        // Create script string
        let source = v8::String::new(scope, script).ok_or("Failed to create script string")?;
        let script = v8::Script::compile(scope, source, None).ok_or("Failed to compile script")?;

        // Run script
        let result = script.run(scope).ok_or("Failed to run script")?;
        let result = result.to_string(scope).ok_or("Failed to convert result to string")?;
        
        Ok(result.to_rust_string_lossy(scope))
    }

    fn set_global(&self, name: &str, value: &str) -> Result<(), Box<dyn Error>> {
        let mut scope = HandleScope::new(&mut self.isolate.clone());
        let context = self.context.open(&mut scope);
        let scope = &mut HandleScope::with_context(&mut scope, context);

        let key = v8::String::new(scope, name).ok_or("Failed to create key string")?;
        let value = v8::String::new(scope, value).ok_or("Failed to create value string")?;

        let global = context.global(scope);
        global.set(scope, key.into(), value.into());

        Ok(())
    }

    fn get_global(&self, name: &str) -> Result<String, Box<dyn Error>> {
        let mut scope = HandleScope::new(&mut self.isolate.clone());
        let context = self.context.open(&mut scope);
        let scope = &mut HandleScope::with_context(&mut scope, context);

        let key = v8::String::new(scope, name).ok_or("Failed to create key string")?;
        let global = context.global(scope);
        
        let value = global.get(scope, key.into()).ok_or("Failed to get global value")?;
        let string_value = value.to_string(scope).ok_or("Failed to convert value to string")?;
        
        Ok(string_value.to_rust_string_lossy(scope))
    }

    fn cleanup(&self) -> Result<(), Box<dyn Error>> {
        // Note: We don't dispose of V8 here since it's managed by the singleton
        Ok(())
    }
}

impl Drop for V8Engine {
    fn drop(&mut self) {
        // Clean up instance-specific resources
        self.isolate.dispose();
    }
} 
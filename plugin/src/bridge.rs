use nice_plug::context::gui::GuiContext;
use nice_plug::prelude::{ParamPtr, Params};
use serde_json::{json, Value};
use std::sync::Arc;

/// Speaks the page's message protocol for any `#[derive(Params)]` struct:
/// `ready`, `begin`, `set` and `end` in; `init` and `values` out. Values are
/// normalized, and the plugin formats the text.
pub struct Bridge {
    /// Keeps the params alive for the pointers below.
    _params: Arc<dyn Params>,
    entries: Vec<(String, ParamPtr)>,
    sent: Vec<f32>,
}

impl Bridge {
    pub fn new(params: Arc<dyn Params>) -> Self {
        let entries: Vec<_> = params.param_map().into_iter().map(|(id, ptr, _group)| (id, ptr)).collect();
        let sent = vec![f32::NAN; entries.len()];
        Self { _params: params, entries, sent }
    }

    fn find(&self, msg: &Value) -> Option<ParamPtr> {
        let id = msg["id"].as_str()?;
        self.entries.iter().find(|(eid, _)| eid == id).map(|(_, ptr)| *ptr)
    }

    pub fn handle(&mut self, msg: &Value, context: &GuiContext, out: &mut Vec<Value>) {
        let ptr = self.find(msg);
        match (msg["type"].as_str().unwrap_or_default(), ptr) {
            ("ready", _) => {
                self.sent.fill(f32::NAN);
                out.push(json!({ "type": "init", "params": self.describe() }));
            }
            ("begin", Some(ptr)) => unsafe { context.raw_begin_set_parameter(ptr) },
            ("set", Some(ptr)) => {
                if let Some(v) = msg["value"].as_f64() {
                    unsafe { context.raw_set_parameter_normalized(ptr, v.clamp(0.0, 1.0) as f32) }
                }
            }
            ("end", Some(ptr)) => unsafe { context.raw_end_set_parameter(ptr) },
            _ => {}
        }
        self.poll(true, out);
    }

    pub fn poll(&mut self, changed: bool, out: &mut Vec<Value>) {
        if !changed {
            return;
        }
        let mut values = Vec::new();
        for ((id, ptr), last) in self.entries.iter().zip(self.sent.iter_mut()) {
            let value = unsafe { ptr.unmodulated_normalized_value() };
            if *last != value {
                *last = value;
                let text = unsafe { ptr.normalized_value_to_string(value, true) };
                values.push(json!({ "id": id, "value": value, "text": text }));
            }
        }
        if !values.is_empty() {
            out.push(json!({ "type": "values", "values": values }));
        }
    }

    fn describe(&self) -> Vec<Value> {
        self.entries
            .iter()
            .map(|(id, ptr)| unsafe {
                let steps = ptr.step_count().unwrap_or(0);
                let options: Vec<String> = (0..=steps)
                    .filter(|_| steps > 0)
                    .map(|i| ptr.normalized_value_to_string(i as f32 / steps as f32, false))
                    .collect();
                json!({
                    "id": id,
                    "name": ptr.name(),
                    "steps": steps,
                    "options": options,
                    "default": ptr.default_normalized_value(),
                })
            })
            .collect()
    }
}

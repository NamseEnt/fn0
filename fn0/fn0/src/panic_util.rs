use std::any::Any;

pub fn panic_payload_string(payload: &Box<dyn Any + Send>) -> String {
    let payload: &(dyn Any + Send) = &**payload;
    if let Some(s) = payload.downcast_ref::<&'static str>() {
        (*s).to_string()
    } else if let Some(s) = payload.downcast_ref::<String>() {
        s.clone()
    } else {
        format!("non-string panic payload (type_id={:?})", payload.type_id())
    }
}

use wasm_bindgen::prelude::*;
use web_sys::{WebSocket, MessageEvent, Event};
use js_sys::Function;

#[wasm_bindgen]
pub struct WsClient {
    ws: WebSocket,
}

#[wasm_bindgen]
impl WsClient {
    #[wasm_bindgen(constructor)]
    pub fn new(url: &str) -> Result<WsClient, JsValue> {
        let ws = WebSocket::new(url)?;
        Ok(WsClient { ws })
    }

    #[wasm_bindgen]
    pub fn send(&self, msg: &str) -> Result<(), JsValue> {
        self.ws.send_with_str(msg)?;
        Ok(())
    }

    #[wasm_bindgen]
    pub fn on_message(&self, callback: Function) -> Result<(), JsValue> {
        let on_message_callback = Closure::wrap(Box::new(move |event: MessageEvent| {
            if let Ok(text) = event.data().dyn_into::<js_sys::JsString>() {
                let _ = callback.call1(&JsValue::NULL, &text);
            }
        }) as Box<dyn FnMut(MessageEvent)>);

        self.ws.set_onmessage(Some(on_message_callback.as_ref().unchecked_ref()));
        on_message_callback.forget();
        
        Ok(())
    }

    #[wasm_bindgen]
    pub fn on_open(&self, callback: Function) -> Result<(), JsValue> {
        let on_open_callback = Closure::wrap(Box::new(move |_event: Event| {
            let _ = callback.call0(&JsValue::NULL);
        }) as Box<dyn FnMut(Event)>);

        self.ws.set_onopen(Some(on_open_callback.as_ref().unchecked_ref()));
        on_open_callback.forget();
        
        Ok(())
    }

    #[wasm_bindgen]
    pub fn on_error(&self, callback: Function) -> Result<(), JsValue> {
        let on_error_callback = Closure::wrap(Box::new(move |event: Event| {
            let _ = callback.call1(&JsValue::NULL, &event.into());
        }) as Box<dyn FnMut(Event)>);

        self.ws.set_onerror(Some(on_error_callback.as_ref().unchecked_ref()));
        on_error_callback.forget();
        
        Ok(())
    }

    #[wasm_bindgen]
    pub fn close(&self) -> Result<(), JsValue> {
        self.ws.close()?;
        Ok(())
    }
}
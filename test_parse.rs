use strut_core::Document;
use serde_json::Value;

fn main() {
    let raw = r#"{"document":{"id":"d69e4f1a-6d1a-4f1a-9d1a-6d1a4f1a9d1a","name":"3D Rolling Dice","artboards":[{"id":"a1","name":"Main","width":960,"height":540,"nodes":["root"]}],"nodes":[{"id":"root","name":"Dice Container","kind":"group","transform":{"translate_x":480,"translate_y":270},"children":[{"id":"cube","name":"Dice Cube","kind":"group","children":[{"id":"f1","name":"Face 1","kind":"group","transform":{"rotate_x":0,"rotate_y":0},"children":[{"id":"f1b","kind":"rect","shape":{"type":"rect","x":-50,"y":-50,"width":100,"height":100,"rx":12},"style":{"fill":"#ffffff","stroke":"#cccccc","stroke_width":2}},{"id":"f1p1","kind":"ellipse","shape":{"type":"ellipse","cx":0,"cy":0,"rx":6,"ry":6},"style":{"fill":"#1a1a1a"}}]}]}]}],"timelines":[{"id":"roll_1","name":"Roll 1","tracks":[{"node_id":"cube","property":"rotate_x","keyframes":[{"time":0,"val":0},{"time":800,"val":1080},{"time":1800,"val":1080}]}]}],"state_machines":[{"id":"sm1","name":"Dice Controller","states":[{"id":"idle","name":"Idle"},{"id":"face_1","name":"Face 1","timeline_id":"roll_1"}]}],"bindings":[],"events":[]}}"#;
    let val: Value = serde_json::from_str(raw).unwrap();
    let doc = val.get("document").unwrap();
    let parsed: Result<Document, _> = serde_json::from_value(doc.clone());
    match parsed {
        Ok(_) => println!("Parsed OK!"),
        Err(e) => println!("Parse error: {}", e),
    }
}

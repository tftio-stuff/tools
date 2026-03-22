use serde_json::Value;
use tftio_cli_common::json::ok_response;

#[test]
fn ok_response_has_expected_shape() {
    let v = ok_response("list", serde_json::json!([]));
    let obj = v.as_object().unwrap();
    assert_eq!(obj.get("ok").unwrap(), &Value::Bool(true));
    assert!(obj.get("data").is_some());
}

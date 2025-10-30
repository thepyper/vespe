use super::Parser;
use serde_json::json;

#[test]
fn test_try_parse_jsonplus_object() {
    //let doc = "{ a:a, b:b, c:1, d:'d', e:\"e\", f, g: 5.0, h:false, i:true }";
    let doc = "{ a:1 }";
    let parser = Parser::new(doc);
    let (value, p_next) = super::_try_parse_jsonplus_object(&parser).unwrap().unwrap();
    // TODO assert
}

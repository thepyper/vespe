use super::Parser;
use serde_json::json;

#[test]
fn test_try_parse_jsonplus_object() {
    let doc = "{ a:a, b:b, c:1, d:'d', e:\"e\", f, g: 5.0, h:false, i:true, j: [1,2,3,4,5], k: { s:s, t:t } }";
    //let doc = "{ x:5 }";
    let parser = Parser::new(doc);
    let (value, p_next) = super::_try_parse_jsonplus_object(&parser).unwrap().unwrap();
    // TODO assert
    //assert_eq!(value.to_string(), "");
    assert!(value.to_string() != "");
}

#[test]
fn test_try_parse_jsonplus_array() {
    let doc = "[ a, 'b', \"c\", 1, 2.2, 3.123, true, false, { a:a, b:b }, [ 1,2,3,4,5,6,7, ] ]";
    //let doc = "{ x:5 }";
    let parser = Parser::new(doc);
    let (value, p_next) = super::_try_parse_jsonplus_array(&parser).unwrap().unwrap();
    // TODO assert
    //assert_eq!(value, Vec::new());
    assert!(!value.is_empty());
}

use super::utils::{create_position, create_range};
use super::*;

#[test]
fn test_position_is_valid() {
    let valid_pos = create_position(0, 1, 1);
    assert!(valid_pos.is_valid());

    let invalid_line_pos = create_position(0, 0, 1);
    assert!(!invalid_line_pos.is_valid());

    let invalid_column_pos = create_position(0, 1, 0);
    assert!(!invalid_column_pos.is_valid());

    let null_pos = Position::null();
    assert!(!null_pos.is_valid());
}

#[test]
fn test_range_is_valid() {
    // Valid range
    let valid_range = create_range(0, 1, 1, 5, 1, 6);
    assert!(valid_range.is_valid());

    // Invalid range: end before begin
    let invalid_offset_range = create_range(5, 1, 6, 0, 1, 1);
    assert!(!invalid_offset_range.is_valid());

    // Invalid range: invalid begin position
    let invalid_begin_range = Range {
        begin: Position::null(),
        end: create_position(5, 1, 6),
    };
    assert!(!invalid_begin_range.is_valid());

    // Invalid range: invalid end position
    let invalid_end_range = Range {
        begin: create_position(0, 1, 1),
        end: Position::null(),
    };
    assert!(!invalid_end_range.is_valid());

    let null_range = Range::null();
    assert!(!null_range.is_valid());
}

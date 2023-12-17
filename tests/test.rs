use funnelparse::{self, ContextError, Parser};

#[test]
fn char_tag_parser_success() {
    let mut input = "abc";
    let res: Result<_, ContextError> = 'a'.parse(&mut input);
    assert_eq!('a', res.unwrap());
    assert_eq!("bc", input);
}

#[test]
fn char_tag_parser_fail() {
    let mut input = "cde";
    let res: Result<_, ContextError> = 'a'.parse(&mut input);
    assert!(res.is_err());
    assert_eq!("cde", input);
}

#[test]
fn slice_tag_parser_success() {
    let mut slice = [1, 2, 3, 4].as_slice();
    let res: Result<_, ContextError> = [1, 2].as_slice().parse(&mut slice);
    assert_eq!([1, 2], res.unwrap());
    assert_eq!([3, 4], slice);
}

#[test]
fn slice_tag_parser_fail_short() {
    let mut slice = [1].as_slice();
    let res: Result<_, ContextError> = [1, 2].as_slice().parse(&mut slice);
    assert!(res.is_err());
    assert_eq!([1], slice);
}

#[test]
fn slice_tag_parser_fail_mismatch() {
    let mut slice = [1, 4, 8].as_slice();
    let res: Result<_, ContextError> = [1, 2].as_slice().parse(&mut slice);
    assert!(res.is_err());
    assert_eq!([1, 4, 8], slice);
}

#[test]
fn const_array_parser_success() {
    let mut slice = [1, 2, 3, 4].as_slice();
    let res: Result<_, ContextError> = [1, 2].parse(&mut slice);
    assert_eq!([1, 2], res.unwrap());
    assert_eq!([3, 4], slice);
}

#[test]
fn tag_eq_parser_success() {
    let mut slice = [1, 2, 3, 4].as_slice();
    let res: Result<_, ContextError> = 1.parse(&mut slice);
    assert_eq!(1, res.unwrap());
    assert_eq!([2, 3, 4], slice);
}

#[test]
fn fnbool_slice_parser_success() {
    let mut slice = [1, 2, 3, 4].as_slice();
    let res: Result<_, ContextError> = (|x| x == 1).parse(&mut slice);
    assert_eq!(1, res.unwrap());
    assert_eq!([2, 3, 4], slice);
}

#[test]
fn fnbool_slice_parser_error() {
    let mut slice = [1, 2, 3, 4].as_slice();
    let res: Result<_, ContextError> = (|x| x == 3).parse(&mut slice);
    assert!(res.is_err());
    assert_eq!([1, 2, 3, 4], slice);
}

#[test]
fn fnbool_str_parser_success() {
    let mut input = "abc";
    let res: Result<_, ContextError> = (|x| x == 'a').parse(&mut input);
    assert_eq!('a', res.unwrap());
    assert_eq!("bc", input);
}

#[test]
fn fnbool_str_parser_error() {
    let mut input = "cde";
    let res: Result<_, ContextError> = (|x| x == 'a').parse(&mut input);
    assert!(res.is_err());
    assert_eq!("cde", input);
}

fn str_option_func(c: char) -> Option<i32> {
    if c == 'a' {
        Some(5)
    } else {
        None
    }
}

#[test]
fn fnoption_str_parser_success() {
    let mut input = "abc";
    let res: Result<_, ContextError> = str_option_func.parse(&mut input);
    assert_eq!(5, res.unwrap());
    assert_eq!("bc", input);
}

#[test]
fn fnoption_str_parser_error() {
    let mut input = "cde";
    let res: Result<_, ContextError> = str_option_func.parse(&mut input);
    assert!(res.is_err());
    assert_eq!("cde", input);
}

fn slice_option_func(val: i32) -> Option<i32> {
    if val == 1 {
        Some(5)
    } else {
        None
    }
}

#[test]
fn fnoption_slice_parser_success() {
    let mut slice = [1, 2, 3, 4].as_slice();
    let res: Result<_, ContextError> = slice_option_func.parse(&mut slice);
    assert_eq!(5, res.unwrap());
    assert_eq!([2, 3, 4], slice);
}

#[test]
fn fnoption_slice_parser_error() {
    let mut slice = [2, 3, 4].as_slice();
    let res: Result<_, ContextError> = slice_option_func.parse(&mut slice);
    assert!(res.is_err());
    assert_eq!([2, 3, 4], slice);
}

#[test]
fn tag_ergonomics() {
    let mut slice = [1, 2, 3, 4].as_slice();

    fn parse_slice(input: &mut &[i32]) -> Result<i32, ContextError> {
        1.parse(input)
    }
    let res = parse_slice(&mut slice);
    assert_eq!(1, res.unwrap());
    assert_eq!([2, 3, 4], slice);
}

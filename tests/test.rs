use std::{collections::HashMap, error::Error, fmt, str::FromStr};

use fabparse::{self, opt, take, take_not, ContextError, Parser};
#[test]
fn char_tag_parser_success() {
    let mut input = "abc";
    let res: Result<_, ContextError> = 'a'.fab(&mut input);
    assert_eq!('a', res.unwrap());
    assert_eq!("bc", input);
}

#[test]
fn char_tag_parser_fail() {
    let mut input = "cde";
    let res: Result<_, ContextError> = 'a'.fab(&mut input);
    assert!(res.is_err());
    assert_eq!("cde", input);
}

#[test]
fn slice_tag_parser_success() {
    let mut slice = [1, 2, 3, 4].as_slice();
    let res: Result<_, ContextError> = [1, 2].as_slice().fab(&mut slice);
    assert_eq!([1, 2], res.unwrap());
    assert_eq!([3, 4], slice);
}

#[test]
fn slice_tag_parser_fail_short() {
    let mut slice = [1].as_slice();
    let res: Result<_, ContextError> = [1, 2].as_slice().fab(&mut slice);
    assert!(res.is_err());
    assert_eq!([1], slice);
}

#[test]
fn slice_tag_parser_fail_mismatch() {
    let mut slice = [1, 4, 8].as_slice();
    let res: Result<_, ContextError> = [1, 2].as_slice().fab(&mut slice);
    assert!(res.is_err());
    assert_eq!([1, 4, 8], slice);
}

#[test]
fn const_array_parser_success() {
    let mut slice = [1, 2, 3, 4].as_slice();
    let res: Result<_, ContextError> = [1, 2].fab(&mut slice);
    assert_eq!([1, 2], res.unwrap());
    assert_eq!([3, 4], slice);
}

#[test]
fn tag_eq_parser_success() {
    let mut slice = [1, 2, 3, 4].as_slice();
    let res: Result<_, ContextError> = 1.fab(&mut slice);
    assert_eq!(1, res.unwrap());
    assert_eq!([2, 3, 4], slice);
}

#[test]
fn fnbool_slice_parser_success() {
    let mut slice = [1, 2, 3, 4].as_slice();
    let res: Result<_, ContextError> = (|x| x == 1).fab(&mut slice);
    assert_eq!(1, res.unwrap());
    assert_eq!([2, 3, 4], slice);
}

#[test]
fn fnbool_slice_parser_error() {
    let mut slice = [1, 2, 3, 4].as_slice();
    let res: Result<_, ContextError> = (|x| x == 3).fab(&mut slice);
    assert!(res.is_err());
    assert_eq!([1, 2, 3, 4], slice);
}

#[test]
fn fnbool_str_parser_success() {
    let mut input = "abc";
    let res: Result<_, ContextError> = (|x| x == 'a').fab(&mut input);
    assert_eq!('a', res.unwrap());
    assert_eq!("bc", input);
}

#[test]
fn fnbool_str_parser_error() {
    let mut input = "cde";
    let res: Result<_, ContextError> = (|x| x == 'a').fab(&mut input);
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
    let res: Result<_, ContextError> = str_option_func.fab(&mut input);
    assert_eq!(5, res.unwrap());
    assert_eq!("bc", input);
}

#[test]
fn fnoption_str_parser_error() {
    let mut input = "cde";
    let res: Result<_, ContextError> = str_option_func.fab(&mut input);
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
    let res: Result<_, ContextError> = slice_option_func.fab(&mut slice);
    assert_eq!(5, res.unwrap());
    assert_eq!([2, 3, 4], slice);
}

#[test]
fn fnoption_slice_parser_error() {
    let mut slice = [2, 3, 4].as_slice();
    let res: Result<_, ContextError> = slice_option_func.fab(&mut slice);
    assert!(res.is_err());
    assert_eq!([2, 3, 4], slice);
}

fn slice_result_func(val: i32) -> Result<i32, <i32 as FromStr>::Err> {
    if val == 1 {
        <i32 as FromStr>::from_str("5")
    } else {
        <i32 as FromStr>::from_str("a")
    }
}

fn str_result_func(c: char) -> Result<i32, <i32 as FromStr>::Err> {
    if c == 'a' {
        <i32 as FromStr>::from_str("5")
    } else {
        <i32 as FromStr>::from_str("a")
    }
}

#[test]
fn fnresult_slice_parser_success() {
    let mut slice = [1, 2, 3, 4].as_slice();
    let res: Result<_, ContextError> = slice_result_func.fab(&mut slice);
    assert_eq!(5, res.unwrap());
    assert_eq!([2, 3, 4], slice);
}

#[test]
fn fnresult_slice_parser_error() {
    let mut slice = [2, 3, 4].as_slice();
    let res: Result<_, ContextError> = slice_result_func.fab(&mut slice);
    assert!(res.is_err());
    assert_eq!([2, 3, 4], slice);
}

#[test]
fn fnresult_str_parser_success() {
    let mut input = "abc";
    let res: Result<_, ContextError> = str_result_func.fab(&mut input);
    assert_eq!(5, res.unwrap());
    assert_eq!("bc", input);
}

#[test]
fn fnresult_str_parser_error() {
    let mut input = "cde";
    let res: Result<_, ContextError> = str_result_func.fab(&mut input);
    assert!(res.is_err());
    assert_eq!("cde", input);
}

#[test]
fn strtag_unicode_success() {
    let mut input = "😀🇷🇺";
    let res: Result<_, ContextError> = '😀'.fab(&mut input);
    assert_eq!('😀', res.unwrap());
    assert_eq!("🇷🇺", input);
}

#[test]
fn tag_ergonomics() {
    let mut slice = [1, 2, 3, 4].as_slice();

    fn parse_slice(input: &mut &[i32]) -> Result<i32, ContextError> {
        1.fab(input)
    }
    let res = parse_slice(&mut slice);
    assert_eq!(1, res.unwrap());
    assert_eq!([2, 3, 4], slice);
}

#[test]
fn map_success() {
    let mut input = "abc";
    let res: Result<_, ContextError> = 'a'.fab_map(|_| 5).fab(&mut input);
    assert_eq!(5, res.unwrap());
    assert_eq!("bc", input);
}

#[test]
fn take_map_success() {
    let mut input = "abc";
    let res: Result<_, ContextError> = take(1).fab_map(|_| 5).fab(&mut input);
    assert_eq!(5, res.unwrap());
    assert_eq!("bc", input);
}

#[test]
fn take_map_fail() {
    let mut input = "abc";
    let res: Result<_, ContextError> = take(4).fab_map(|_| 5).fab(&mut input);
    assert!(res.is_err());
    assert_eq!("abc", input);
}

#[test]
fn try_success() {
    let mut input = "abc";
    let res: Result<_, ContextError> = take(1).fab_map(|_| Some(5)).fab_try().fab(&mut input);
    assert_eq!(5, res.unwrap());
    assert_eq!("bc", input);
}

#[test]
fn try_parser_inner_fail() {
    let mut input = "abc";
    let res: Result<_, ContextError> = take(4).fab_map(|_| Some(5)).fab_try().fab(&mut input);
    assert!(res.is_err());
    assert_eq!("abc", input);
}

#[test]
fn try_parser_none_fail() {
    let mut input = "abc";
    let res: Result<_, ContextError> = take(1).fab_map(|_| None::<i32>).fab_try().fab(&mut input);
    assert!(res.is_err());
    assert_eq!("abc", input);
}

#[derive(Debug)]
struct TestError;

impl fmt::Display for TestError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "TestError")
    }
}

impl Error for TestError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}

#[test]
fn try_result_success() {
    let mut input = "abc";
    let res: Result<_, ContextError> = take(1)
        .fab_map(|_| Ok::<_, TestError>(5))
        .fab_try()
        .fab(&mut input);
    assert_eq!(5, res.unwrap());
    assert_eq!("bc", input);
}

#[test]
fn try_result_parser_inner_fail() {
    let mut input = "abc";
    let res: Result<_, ContextError> = take(4)
        .fab_map(|_| Ok::<_, TestError>(5))
        .fab_try()
        .fab(&mut input);
    assert!(res.is_err());
    assert_eq!("abc", input);
}

#[test]
fn try_result_parser_none_fail() {
    let mut input = "abc";
    let res: Result<_, ContextError> = take(4)
        .fab_map(|_| Err::<i32, _>(TestError))
        .fab_try()
        .fab(&mut input);
    assert!(res.is_err());
    assert_eq!("abc", input);
}

#[test]
fn try_result_trait_method_success() {
    let mut input = "abc";
    let res: Result<_, ContextError> = take(1)
        .fab_map(|_| Ok::<_, TestError>(5))
        .fab_try()
        .fab(&mut input);
    assert_eq!(5, res.unwrap());
    assert_eq!("bc", input);
}

#[test]
fn map_trait_method_success() {
    let mut input = "abc";
    let res: Result<_, ContextError> = take(1).fab_map(|_| 5).fab(&mut input);
    assert_eq!(5, res.unwrap());
    assert_eq!("bc", input);
}

#[test]
fn try_map_option_success() {
    let mut input = "abc";
    let res: Result<_, ContextError> = take(1).fab_try_map(|_| Some(5)).fab(&mut input);
    assert_eq!(5, res.unwrap());
    assert_eq!("bc", input);
}

#[test]
fn try_map_option_fail() {
    let mut input = "abc";
    let res: Result<_, ContextError> = take(1).fab_try_map(|_| None::<i32>).fab(&mut input);
    assert!(res.is_err());
    assert_eq!("abc", input);
}

#[test]
fn try_map_option_parser_fail() {
    let mut input = "abc";
    let res: Result<_, ContextError> = take(4).fab_try_map(|_| Some(5)).fab(&mut input);
    assert!(res.is_err());
    assert_eq!("abc", input);
}

#[test]
fn try_map_result_success() {
    let mut input = "abc";
    let res: Result<_, ContextError> = take(1)
        .fab_try_map(|_| Ok::<_, TestError>(5))
        .fab(&mut input);
    assert_eq!(5, res.unwrap());
    assert_eq!("bc", input);
}

#[test]
fn try_map_result_fail() {
    let mut input = "abc";
    let res: Result<_, ContextError> = take(1)
        .fab_try_map(|_| Err::<i32, _>(TestError))
        .fab(&mut input);
    assert!(res.is_err());
    assert_eq!("abc", input);
}

#[test]
fn try_map_option_result_fail() {
    let mut input = "abc";
    let res: Result<_, ContextError> = take(4)
        .fab_try_map(|_| Ok::<_, TestError>(5))
        .fab(&mut input);
    assert!(res.is_err());
    assert_eq!("abc", input);
}

#[test]
fn str_success() {
    let mut input = "abc";
    let res: Result<_, ContextError> = "a".fab(&mut input);
    assert_eq!("a", res.unwrap());
    assert_eq!("bc", input);
    let mut input = "abc";
    let res: Result<_, ContextError> = "abc".fab(&mut input);
    assert_eq!("abc", res.unwrap());
    assert_eq!("", input);
}

#[test]
fn str_fail() {
    let mut input = "abc";
    let res: Result<_, ContextError> = "ad".fab(&mut input);
    assert!(res.is_err());
    assert_eq!("abc", input);
}

#[test]
fn opt_success() {
    let mut input = "abc";
    let res: Result<_, ContextError> = opt("a").fab(&mut input);
    assert_eq!(Some("a"), res.unwrap());
    assert_eq!("bc", input);
    let mut input = "abc";
    let res: Result<_, ContextError> = opt("b").fab(&mut input);
    assert_eq!(None, res.unwrap());
    assert_eq!("abc", input);
}

#[test]
fn tuple_success() {
    let mut input = "abc";
    let res: Result<_, ContextError> = ("a", "b").fab(&mut input);
    assert_eq!(("a", "b"), res.unwrap());
    assert_eq!("c", input);
}

#[test]
fn tuple_fail() {
    let mut input = "abc";
    let res: Result<_, ContextError> = ("b", "a").fab(&mut input);
    assert!(res.is_err());
    assert_eq!("abc", input);
}

#[test]
fn range_success() {
    let mut input = "abc";
    let res: Result<_, ContextError> = ('a'..='z').fab(&mut input);
    assert_eq!('a', res.unwrap());
    assert_eq!("bc", input);
}

#[test]
fn range_fail() {
    let mut input = "abc";
    let res: Result<_, ContextError> = ('b'..='z').fab(&mut input);
    assert!(res.is_err());
    assert_eq!("abc", input);
}

#[test]
fn take_not_success() {
    let mut input = "abc";
    let res: Result<_, ContextError> = take_not("b").fab(&mut input);
    assert_eq!('a', res.unwrap());
    assert_eq!("bc", input);
}

#[test]
fn take_not_fail() {
    let mut input = "abc";
    let res: Result<_, ContextError> = take_not("a").fab(&mut input);
    assert!(res.is_err());
    assert_eq!("abc", input);
}

#[test]
fn take_not_empty() {
    let mut input = "";
    let res: Result<_, ContextError> = take_not("a").fab(&mut input);
    assert!(res.is_err());
    assert_eq!("", input);
}

#[test]
fn value_success() {
    let mut input = "abc";
    let res: Result<_, ContextError> = "a".fab_value(5).fab(&mut input);
    assert_eq!(5, res.unwrap());
    assert_eq!("bc", input);
}

#[test]
fn value_fail() {
    let mut input = "abc";
    let res: Result<_, ContextError> = "b".fab_value(5).fab(&mut input);
    assert!(res.is_err());
    assert_eq!("abc", input);
}

#[test]
fn repeat_success() {
    let mut input = "aac";
    let res: Result<_, ContextError> = 'a'.fab_repeat().fab(&mut input);
    assert_eq!(vec!['a', 'a'], res.unwrap());
    assert_eq!("c", input);
}

#[test]
fn repeat_min_success() {
    let mut input = "aac";
    let res: Result<_, ContextError> = 'a'.fab_repeat().min(2).fab(&mut input);
    assert_eq!(vec!['a', 'a'], res.unwrap());
    assert_eq!("c", input);
}

#[test]
fn repeat_min_fail() {
    let mut input = "aac";
    let res: Result<_, ContextError> = 'a'.fab_repeat().min(3).fab(&mut input);
    assert!(res.is_err());
    assert_eq!("aac", input);
}

#[test]
fn repeat_max_success() {
    let mut input = "aac";
    let res: Result<_, ContextError> = 'a'.fab_repeat().max(3).fab(&mut input);
    assert_eq!(vec!['a', 'a'], res.unwrap());
    assert_eq!("c", input);
}

#[test]
fn repeat_max_fail() {
    let mut input = "aac";
    let res: Result<_, ContextError> = 'a'.fab_repeat().max(2).fab(&mut input);
    assert!(res.is_err());
    assert_eq!("aac", input);
}

fn char_num<'a>(input: &mut &'a str) -> Result<(char, u32), ContextError> {
    ('a'..='z', ('0'..='9').fab_try_map(|c: char| c.to_digit(10))).fab(input)
}

fn reducer(map: &mut HashMap<char, u32>, val: (char, u32)) {
    map.insert(val.0, val.1);
}
#[test]
fn repeat_reduce_hashmap_success() {
    let mut input = "a1b2c3";
    let res: Result<_, ContextError> = char_num
        .fab_repeat()
        .reduce(HashMap::new(), reducer)
        .fab(&mut input);
    let expected_res: HashMap<char, u32> = [('a',1),('b',2),('c',3)].into_iter().collect();
    assert_eq!(expected_res, res.unwrap());
    assert_eq!("", input);
}

#[test]
fn repeat_reduce_hashmap_lambdas() {
    let mut input = "a1b2c3";
    let res: Result<_, ContextError> = ('a'..='z', ('0'..='9').fab_try_map(|c: char| c.to_digit(10)))
        .fab_repeat()
        .reduce(HashMap::new(), |state: &mut HashMap<char, u32>, val: (char, u32)| {state.insert(val.0, val.1);} )
        .fab(&mut input);
    let expected_res: HashMap<char, u32> = [('a',1),('b',2),('c',3)].into_iter().collect();
    assert_eq!(expected_res, res.unwrap());
    assert_eq!("", input);
}
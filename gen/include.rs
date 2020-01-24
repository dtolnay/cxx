static HEADER: &str = include_str!("include/cxxbridge.h");

pub fn get(guard: &str) -> &'static str {
    let ifndef = format!("#ifndef {}\n", guard);
    let endif = format!("#endif // {}\n", guard);
    let begin = HEADER.find(&ifndef);
    let end = HEADER.find(&endif);
    if let (Some(begin), Some(end)) = (begin, end) {
        &HEADER[begin..end + endif.len()]
    } else {
        panic!("not found in cxxbridge.h header: {}", guard)
    }
}

pub fn get_full_cxxbridge() -> &'static str {
	return HEADER
} 
// XML parser

#[derive(Clone, Debug, PartialEq, Eq)]
struct Element {
    name: String,                      // identifier at the start of each tag
    attributes: Vec<(String, String)>, // (identifier, value) for the attributes
    children: Vec<Element>,            // list of child elements that look exactly the same
}

fn the_letter_a(input: &str) -> Result<(&str, ()), &str> {
    match input.chars().next() {
        Some('a') => Ok((&input['a'.len_utf8()..], ())),
        _ => Err(input),
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}

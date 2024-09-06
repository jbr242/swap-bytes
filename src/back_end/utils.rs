use std::collections::HashSet;
use regex::Regex;

const ALLOWED_TOPICS: [&str; 4] = ["chat", "movies", "books", "music"];

pub fn split_string(input: &str) -> Vec<String> {
    let re = Regex::new(r#""([^"]*)"|\S+"#).unwrap();
    re.captures_iter(input)
        .map(|cap| cap.get(0).unwrap().as_str().to_string())
        .collect()
}

pub fn check_topic(topic: &str) -> bool {
    let allowed_topics: HashSet<&str> = ALLOWED_TOPICS.iter().cloned().collect();
    allowed_topics.contains(topic)
}

pub fn print_allowed_topics() {
    let allowed_topics: HashSet<&str> = ALLOWED_TOPICS.iter().cloned().collect();
    println!("Allowed topics: {}", allowed_topics.iter().cloned().collect::<Vec<&str>>().join(", "));
}

use std::collections::HashSet;
use once_cell::sync::Lazy;

static INAPPROPRIATE_WORDS: Lazy<HashSet<&'static str>> = Lazy::new(|| {
    let mut set = HashSet::new();
    // Common inappropriate words (this is a basic list, expand as needed)
    set.insert("anjing");
    set.insert("babi");
    set.insert("bangsat");
    set.insert("kontol");
    set.insert("memek");
    set.insert("ngentot");
    set.insert("jancok");
    set.insert("fuck");
    set.insert("shit");
    set.insert("dick");
    set.insert("bitch");
    set.insert("bastard");
    set.insert("asshole");
    // Add more words as needed
    set
});

pub fn contains_inappropriate_content(text: &str) -> bool {
    let text_lower = text.to_lowercase();
    INAPPROPRIATE_WORDS.iter().any(|&word| text_lower.contains(word))
}

pub fn filter_message(text: &str) -> String {
    let mut filtered_text = text.to_string();
    for word in INAPPROPRIATE_WORDS.iter() {
        let replacement = "*".repeat(word.len());
        filtered_text = filtered_text.replace(word, &replacement);
    }
    filtered_text
} 
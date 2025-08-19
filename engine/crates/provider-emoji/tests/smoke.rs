use lancea_provider_emoji::EmojiProvider;
use lancea_model::Provider;

#[test]
fn can_create_provider() {
    let provider = EmojiProvider::new().expect("Failed to create emoji provider");
    assert_eq!(provider.id(), "emoji");
}

#[test]
fn search_returns_results_for_joy() {
    let provider = EmojiProvider::new().expect("Failed to create emoji provider");
    let results = provider.search("joy");
    
    assert!(!results.is_empty());
    assert_eq!(results[0].title, "Face with Tears of Joy");
    assert_eq!(results[0].provider_id, "emoji");
}

#[test]
fn search_handles_emoji_prefix() {
    let provider = EmojiProvider::new().expect("Failed to create emoji provider");
    let results = provider.search("/emoji joy");
    
    assert!(!results.is_empty());
    assert_eq!(results[0].title, "Face with Tears of Joy");
}

#[test]
fn search_handles_em_prefix() {
    let provider = EmojiProvider::new().expect("Failed to create emoji provider");
    let results = provider.search("/em smile");
    
    assert!(!results.is_empty());
    let smile_result = results.iter().find(|r| r.title.contains("Smiling"));
    assert!(smile_result.is_some());
}

#[test]
fn search_is_case_insensitive() {
    let provider = EmojiProvider::new().expect("Failed to create emoji provider");
    let results_lower = provider.search("smile");
    let results_upper = provider.search("SMILE");
    
    assert!(!results_lower.is_empty());
    assert!(!results_upper.is_empty());
    assert_eq!(results_lower.len(), results_upper.len());
}

#[test]
fn search_by_shortcode() {
    let provider = EmojiProvider::new().expect("Failed to create emoji provider");
    let results = provider.search("joy");
    
    assert!(!results.is_empty());
    assert_eq!(results[0].score, 1.0); // Exact shortcode match should have highest score
}

#[test]
fn search_by_keyword() {
    let provider = EmojiProvider::new().expect("Failed to create emoji provider");
    let results = provider.search("laugh");
    
    assert!(!results.is_empty());
    let joy_result = results.iter().find(|r| r.title == "Face with Tears of Joy");
    assert!(joy_result.is_some());
}

#[test]
fn preview_returns_data_for_valid_key() {
    let provider = EmojiProvider::new().expect("Failed to create emoji provider");
    let preview = provider.preview("emoji:joy");
    
    assert!(preview.is_some());
    let preview = preview.unwrap();
    assert_eq!(preview.preview_kind, "card");
    assert!(preview.data.get("glyph").is_some());
    assert!(preview.data.get("title").is_some());
}

#[test]
fn preview_returns_none_for_invalid_key() {
    let provider = EmojiProvider::new().expect("Failed to create emoji provider");
    let preview = provider.preview("emoji:nonexistent");
    
    assert!(preview.is_none());
}

#[test]
fn execute_copy_glyph_succeeds_for_valid_key() {
    let provider = EmojiProvider::new().expect("Failed to create emoji provider");
    let result = provider.execute("copy_glyph", "emoji:joy");
    
    assert!(result);
}

#[test]
fn execute_copy_shortcode_succeeds_for_valid_key() {
    let provider = EmojiProvider::new().expect("Failed to create emoji provider");
    let result = provider.execute("copy_shortcode", "emoji:joy");
    
    assert!(result);
}

#[test]
fn execute_fails_for_invalid_action() {
    let provider = EmojiProvider::new().expect("Failed to create emoji provider");
    let result = provider.execute("invalid_action", "emoji:joy");
    
    assert!(!result);
}

#[test]
fn execute_fails_for_invalid_key() {
    let provider = EmojiProvider::new().expect("Failed to create emoji provider");
    let result = provider.execute("copy_glyph", "emoji:nonexistent");
    
    assert!(!result);
}

#[test]
fn search_results_are_sorted_by_score() {
    let provider = EmojiProvider::new().expect("Failed to create emoji provider");
    let results = provider.search("smile");
    
    assert!(results.len() > 1);
    // Verify scores are in descending order
    for i in 1..results.len() {
        assert!(results[i-1].score >= results[i].score);
    }
}

#[test]
fn empty_search_returns_low_score_results() {
    let provider = EmojiProvider::new().expect("Failed to create emoji provider");
    let results = provider.search("");
    
    assert!(!results.is_empty());
    // All results should have low score (0.1) for empty query
    for result in &results {
        assert_eq!(result.score, 0.1);
    }
}
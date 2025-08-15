use lancea_model::ResolvedCommand;

pub struct CommandRegistry;

impl CommandRegistry {
    pub fn new() -> Self {
        Self
    }

    pub fn resolve(&self, text: &str) -> ResolvedCommand {
        let trimmed = text.trim();

        if trimmed.starts_with("/emoji") || trimmed.starts_with("/em") {
            return ResolvedCommand {
                matched: true,
                providerId: Some("emoji".to_string()),
                commandId: Some("emoji".to_string()),
                intent: None,
                reason: Some("slash-command".into()),
            };
        } else {
            return ResolvedCommand {
                matched: false,
                providerId: None,
                commandId: None,
                intent: None,
                reason: None,
            };
        }
    }
}

use lancea_model::ResolvedCommand;

pub struct CommandRegistry;

impl CommandRegistry {
    pub fn new() -> Self {
        Self
    }

    pub fn resolve(&self, text: &str) -> ResolvedCommand {
        dbg!("[Registry#resolve] - query resolving with: {}", &text);
        let trimmed = text.trim();

        if trimmed.starts_with("/emoji") || trimmed.starts_with("/em") {
            dbg!("[Registry#resolve] - query found with: {}", &trimmed);
            return ResolvedCommand {
                matched: true,
                provider_id: Some("emoji".to_string()),
                command_id: Some("emoji".to_string()),
                intent: None,
                reason: Some("slash-command".into()),
            };
        } else if trimmed.starts_with("/apps") || trimmed.starts_with("/ap") {
            return ResolvedCommand {
                matched: true,
                provider_id: Some("apps".to_string()),
                command_id: Some("apps".to_string()),
                intent: None,
                reason: Some("slash-command".into()),
            };
        } else {
            return ResolvedCommand {
                matched: false,
                provider_id: None,
                command_id: None,
                intent: None,
                reason: None,
            };
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_emoji_aliases() {
        let registry = CommandRegistry::new();
        let resolved = registry.resolve("/emoji laugh");

        assert!(resolved.matched);
        assert_eq!(resolved.provider_id, Some("emoji".to_string()));
        assert_eq!(resolved.command_id, Some("emoji".to_string()));
        assert_eq!(resolved.reason, Some("slash-command".into()));

        let resolved = registry.resolve("just some text");
        assert!(!resolved.matched);
        assert!(resolved.provider_id.is_none());
        assert!(resolved.command_id.is_none());
    }

    #[test]
    fn test_apps_aliases() {
        let registry = CommandRegistry::new();
        let resolved = registry.resolve("/apps spotify");

        assert!(resolved.matched);
        assert_eq!(resolved.provider_id, Some("apps".to_string()));
        assert_eq!(resolved.command_id, Some("apps".to_string()));
        assert_eq!(resolved.reason, Some("slash-command".into()));

        let resolved = registry.resolve("just some text");
        assert!(!resolved.matched);
        assert!(resolved.provider_id.is_none());
        assert!(resolved.command_id.is_none());
    }
}

use clap::Parser;

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn verify_cli() {
        use crate::Cli;
        Cli::command().debug_assert();
    }

    #[test]
    fn test_help_message() {
        use crate::Cli;
        let help_message = Cli::command().render_help().to_string();
        
        // Verify our help message contains key information
        assert!(help_message.contains("tinker"));
        assert!(help_message.contains("A craftsperson's browser"));
        assert!(help_message.contains("--headless"));
        assert!(help_message.contains("--mqtt-url"));
        assert!(help_message.contains("--debug"));
    }

    #[test]
    fn test_default_values() {
        use crate::Cli;
        let args = Cli::parse_from(["tinker"]);
        
        assert!(!args.headless);
        assert_eq!(args.mqtt_url, "mqtt://localhost:1883");
        assert!(!args.debug);
    }

    #[test]
    fn test_custom_values() {
        use crate::Cli;
        let args = Cli::parse_from([
            "tinker",
            "--headless",
            "--mqtt-url",
            "mqtt://test:1883",
            "--debug"
        ]);
        
        assert!(args.headless);
        assert_eq!(args.mqtt_url, "mqtt://test:1883");
        assert!(args.debug);
    }
} 

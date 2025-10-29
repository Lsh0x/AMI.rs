//! Credential Report Domain Operations - Pure Functions

use super::model::*;

/// Pure domain operations for credential reports
pub mod credential_report_operations {
    use super::*;

    /// Generate CSV report from user data (pure function)
    pub fn generate_csv_report(users: Vec<String>) -> String {
        let mut csv = String::from("user,arn,created_date\n");

        for user in users {
            csv.push_str(&format!(
                "{},arn:aws:iam::123456789012:user/{},2024-01-01T00:00:00Z\n",
                user, user
            ));
        }

        csv
    }

    /// Parse CSV report (pure function)
    pub fn parse_csv_report(csv_content: &str) -> Vec<(String, String, String)> {
        csv_content
            .lines()
            .skip(1) // Skip header
            .filter(|line| !line.is_empty())
            .filter_map(|line| {
                let parts: Vec<&str> = line.split(',').collect();
                if parts.len() >= 3 {
                    Some((
                        parts[0].to_string(),
                        parts[1].to_string(),
                        parts[2].to_string(),
                    ))
                } else {
                    None
                }
            })
            .collect()
    }

    /// Calculate report status based on elapsed time (pure function)
    pub fn calculate_report_status(
        _start_time: chrono::DateTime<chrono::Utc>,
        elapsed_seconds: i64,
    ) -> CredentialReportStatus {
        if elapsed_seconds < 5 {
            CredentialReportStatus::InProgress
        } else if elapsed_seconds < 300 {
            // 5 minutes
            CredentialReportStatus::Complete
        } else {
            CredentialReportStatus::Failed
        }
    }

    /// Check if a credential report needs regeneration (pure function)
    pub fn needs_regeneration(
        generated_time: chrono::DateTime<chrono::Utc>,
        max_age_hours: i64,
    ) -> bool {
        let now = chrono::Utc::now();
        let age = now.signed_duration_since(generated_time);
        age.num_hours() >= max_age_hours
    }

    /// Validate report format (pure function)
    pub fn is_valid_format(format: &str) -> bool {
        matches!(format, "text/csv" | "application/json")
    }

    /// Calculate report size in bytes (pure function)
    pub fn calculate_report_size(report_content: &[u8]) -> usize {
        report_content.len()
    }

    /// Check if report is empty (pure function)
    pub fn is_empty_report(report_content: &[u8]) -> bool {
        report_content.is_empty() || report_content.len() < 20 // Less than header size
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;
    use credential_report_operations::*;

    #[test]
    fn test_generate_csv_report_empty() {
        let csv = generate_csv_report(vec![]);
        assert_eq!(csv, "user,arn,created_date\n");
    }

    #[test]
    fn test_generate_csv_report_single_user() {
        let csv = generate_csv_report(vec!["alice".to_string()]);
        assert!(csv.contains("user,arn,created_date"));
        assert!(csv.contains("alice"));
    }

    #[test]
    fn test_generate_csv_report_multiple_users() {
        let csv = generate_csv_report(vec![
            "alice".to_string(),
            "bob".to_string(),
            "charlie".to_string(),
        ]);

        assert!(csv.contains("alice"));
        assert!(csv.contains("bob"));
        assert!(csv.contains("charlie"));

        let lines: Vec<&str> = csv.lines().collect();
        assert_eq!(lines.len(), 4); // Header + 3 users
    }

    #[test]
    fn test_parse_csv_report_empty() {
        let parsed = parse_csv_report("");
        assert_eq!(parsed.len(), 0);
    }

    #[test]
    fn test_parse_csv_report_header_only() {
        let parsed = parse_csv_report("user,arn,created_date\n");
        assert_eq!(parsed.len(), 0);
    }

    #[test]
    fn test_parse_csv_report_valid() {
        let csv = "user,arn,created_date\nalice,arn:aws:iam::123:user/alice,2024-01-01\nbob,arn:aws:iam::123:user/bob,2024-01-02\n";
        let parsed = parse_csv_report(csv);

        assert_eq!(parsed.len(), 2);
        assert_eq!(parsed[0].0, "alice");
        assert_eq!(parsed[1].0, "bob");
    }

    #[test]
    fn test_parse_csv_report_malformed() {
        let csv = "user,arn,created_date\nalice,incomplete\n";
        let parsed = parse_csv_report(csv);
        assert_eq!(parsed.len(), 0); // Malformed line is filtered out
    }

    #[test]
    fn test_calculate_report_status_in_progress() {
        let start = chrono::Utc::now();
        let status = calculate_report_status(start, 3);
        assert_eq!(status, CredentialReportStatus::InProgress);
    }

    #[test]
    fn test_calculate_report_status_complete() {
        let start = chrono::Utc::now();
        let status = calculate_report_status(start, 10);
        assert_eq!(status, CredentialReportStatus::Complete);
    }

    #[test]
    fn test_calculate_report_status_failed() {
        let start = chrono::Utc::now();
        let status = calculate_report_status(start, 400);
        assert_eq!(status, CredentialReportStatus::Failed);
    }

    #[test]
    fn test_needs_regeneration_fresh() {
        let generated_time = chrono::Utc::now();
        assert!(!needs_regeneration(generated_time, 4));
    }

    #[test]
    fn test_needs_regeneration_old() {
        let generated_time = chrono::Utc::now() - Duration::hours(5);
        assert!(needs_regeneration(generated_time, 4));
    }

    #[test]
    fn test_needs_regeneration_exact_boundary() {
        let generated_time = chrono::Utc::now() - Duration::hours(4);
        assert!(needs_regeneration(generated_time, 4));
    }

    #[test]
    fn test_is_valid_format() {
        assert!(is_valid_format("text/csv"));
        assert!(is_valid_format("application/json"));
        assert!(!is_valid_format("text/plain"));
        assert!(!is_valid_format("application/xml"));
        assert!(!is_valid_format(""));
    }

    #[test]
    fn test_calculate_report_size() {
        assert_eq!(calculate_report_size(b""), 0);
        assert_eq!(calculate_report_size(b"test"), 4);
        assert_eq!(calculate_report_size(&[1, 2, 3, 4, 5]), 5);
    }

    #[test]
    fn test_is_empty_report() {
        assert!(is_empty_report(b""));
        assert!(is_empty_report(b"short"));
        assert!(!is_empty_report(b"user,arn,created_date\n"));
        assert!(!is_empty_report(&[0; 100]));
    }

    #[test]
    fn test_report_status_enum_equality() {
        assert_eq!(
            CredentialReportStatus::InProgress,
            CredentialReportStatus::InProgress
        );
        assert_eq!(
            CredentialReportStatus::Complete,
            CredentialReportStatus::Complete
        );
        assert_eq!(
            CredentialReportStatus::Failed,
            CredentialReportStatus::Failed
        );
        assert_ne!(
            CredentialReportStatus::InProgress,
            CredentialReportStatus::Complete
        );
    }
}

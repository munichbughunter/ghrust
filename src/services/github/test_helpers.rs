// Helper functions for tests that simplify test data creation
use crate::models::github::{
    CopilotDotcomChat, CopilotDotcomPullRequests, CopilotIdeChat, CopilotIdeCodeCompletions,
    CopilotMetrics, Editor, Language, Model, Repository,
};
use anyhow::Result;
use chrono::Utc;

/// Create test metrics suitable for enterprise testing
pub fn create_test_metrics() -> CopilotMetrics {
    CopilotMetrics {
        date: "2023-03-01".to_string(),
        total_active_users: Some(1000),
        total_engaged_users: Some(800),
        copilot_ide_code_completions: Some(CopilotIdeCodeCompletions {
            total_engaged_users: 600,
            languages: Some(vec![Language {
                name: "Rust".to_string(),
                total_engaged_users: 300,
                total_code_suggestions: Some(5000),
                total_code_acceptances: Some(2500),
                total_code_lines_suggested: Some(10000),
                total_code_lines_accepted: Some(5000),
            }]),
            editors: Some(vec![Editor {
                name: "VS Code".to_string(),
                total_engaged_users: 550,
                models: None,
            }]),
        }),
        copilot_ide_chat: Some(CopilotIdeChat {
            total_engaged_users: 400,
            editors: Some(vec![Editor {
                name: "VS Code".to_string(),
                total_engaged_users: 375,
                models: None,
            }]),
        }),
        copilot_dotcom_chat: Some(CopilotDotcomChat {
            total_engaged_users: 300,
            models: Some(vec![Model {
                name: "GPT-4".to_string(),
                is_custom_model: false,
                custom_model_training_date: None,
                total_engaged_users: 290,
                languages: None,
                total_chats: Some(500),
                total_chat_insertion_events: Some(300),
                total_chat_copy_events: Some(200),
                total_pr_summaries_created: None,
            }]),
        }),
        copilot_dotcom_pull_requests: Some(CopilotDotcomPullRequests {
            total_engaged_users: 200,
            repositories: Some(vec![Repository {
                name: "test-repo".to_string(),
                total_engaged_users: 180,
                models: vec![Model {
                    name: "GPT-4".to_string(),
                    is_custom_model: false,
                    custom_model_training_date: None,
                    total_engaged_users: 170,
                    languages: None,
                    total_chats: None,
                    total_chat_insertion_events: None,
                    total_chat_copy_events: None,
                    total_pr_summaries_created: Some(50),
                }],
            }]),
        }),
    }
}

/// Create test metrics suitable for team testing (smaller numbers)
pub fn create_test_team_metrics() -> CopilotMetrics {
    CopilotMetrics {
        date: "2023-03-01".to_string(),
        total_active_users: Some(150),
        total_engaged_users: Some(120),
        copilot_ide_code_completions: Some(CopilotIdeCodeCompletions {
            total_engaged_users: 90,
            languages: Some(vec![Language {
                name: "Rust".to_string(),
                total_engaged_users: 45,
                total_code_suggestions: Some(750),
                total_code_acceptances: Some(375),
                total_code_lines_suggested: Some(1500),
                total_code_lines_accepted: Some(750),
            }]),
            editors: Some(vec![Editor {
                name: "VS Code".to_string(),
                total_engaged_users: 82,
                models: None,
            }]),
        }),
        copilot_ide_chat: Some(CopilotIdeChat {
            total_engaged_users: 60,
            editors: Some(vec![Editor {
                name: "VS Code".to_string(),
                total_engaged_users: 56,
                models: None,
            }]),
        }),
        copilot_dotcom_chat: Some(CopilotDotcomChat {
            total_engaged_users: 45,
            models: Some(vec![Model {
                name: "GPT-4".to_string(),
                is_custom_model: false,
                custom_model_training_date: None,
                total_engaged_users: 43,
                languages: None,
                total_chats: Some(75),
                total_chat_insertion_events: Some(45),
                total_chat_copy_events: Some(30),
                total_pr_summaries_created: None,
            }]),
        }),
        copilot_dotcom_pull_requests: Some(CopilotDotcomPullRequests {
            total_engaged_users: 30,
            repositories: Some(vec![Repository {
                name: "test-repo".to_string(),
                total_engaged_users: 27,
                models: vec![Model {
                    name: "GPT-4".to_string(),
                    is_custom_model: false,
                    custom_model_training_date: None,
                    total_engaged_users: 25,
                    languages: None,
                    total_chats: None,
                    total_chat_insertion_events: None,
                    total_chat_copy_events: None,
                    total_pr_summaries_created: Some(10),
                }],
            }]),
        }),
    }
}

/// Create test metrics focused on IDE chat features for metrics calculation
pub fn create_chat_metrics() -> CopilotMetrics {
    CopilotMetrics {
        date: "2023-03-01".to_string(),
        total_active_users: Some(100),
        total_engaged_users: Some(80),
        copilot_ide_code_completions: None,
        copilot_ide_chat: Some(CopilotIdeChat {
            total_engaged_users: 80,
            editors: Some(vec![
                Editor {
                    name: "VS Code".to_string(),
                    total_engaged_users: 75,
                    models: Some(vec![
                        Model {
                            name: "GPT-4".to_string(),
                            is_custom_model: false,
                            custom_model_training_date: None,
                            total_engaged_users: 70,
                            languages: None,
                            total_chats: Some(137),
                            total_chat_insertion_events: Some(39),
                            total_chat_copy_events: Some(44),
                            total_pr_summaries_created: None,
                        },
                        Model {
                            name: "GPT-3.5".to_string(),
                            is_custom_model: false,
                            custom_model_training_date: None,
                            total_engaged_users: 65,
                            languages: None,
                            total_chats: Some(298),
                            total_chat_insertion_events: Some(0),
                            total_chat_copy_events: Some(0),
                            total_pr_summaries_created: None,
                        },
                    ]),
                },
                Editor {
                    name: "IntelliJ".to_string(),
                    total_engaged_users: 5,
                    models: Some(vec![
                        Model {
                            name: "GPT-4".to_string(),
                            is_custom_model: false,
                            custom_model_training_date: None,
                            total_engaged_users: 5,
                            languages: None,
                            total_chats: Some(44),
                            total_chat_insertion_events: Some(0),
                            total_chat_copy_events: Some(0),
                            total_pr_summaries_created: None,
                        },
                        Model {
                            name: "GPT-3.5".to_string(),
                            is_custom_model: false,
                            custom_model_training_date: None,
                            total_engaged_users: 5,
                            languages: None,
                            total_chats: Some(51),
                            total_chat_insertion_events: Some(0),
                            total_chat_copy_events: Some(0),
                            total_pr_summaries_created: None,
                        },
                    ]),
                },
            ]),
        }),
        copilot_dotcom_chat: None,
        copilot_dotcom_pull_requests: None,
    }
}

/// Create a realistic API response with the current date
pub fn create_mock_api_response() -> Result<Vec<CopilotMetrics>> {
    let now = Utc::now();
    let date = now.format("%Y-%m-%d").to_string();

    Ok(vec![CopilotMetrics {
        date,
        total_active_users: Some(100),
        total_engaged_users: Some(50),
        copilot_ide_code_completions: Some(CopilotIdeCodeCompletions {
            total_engaged_users: 30,
            languages: Some(vec![Language {
                name: "Python".to_string(),
                total_engaged_users: 20,
                total_code_suggestions: Some(1000),
                total_code_acceptances: Some(800),
                total_code_lines_suggested: Some(5000),
                total_code_lines_accepted: Some(4000),
            }]),
            editors: None,
        }),
        copilot_ide_chat: Some(CopilotIdeChat {
            total_engaged_users: 20,
            editors: Some(vec![Editor {
                name: "VS Code".to_string(),
                total_engaged_users: 15,
                models: Some(vec![Model {
                    name: "gpt-4".to_string(),
                    total_engaged_users: 10,
                    total_chats: Some(100),
                    total_chat_copy_events: Some(50),
                    total_chat_insertion_events: Some(30),
                    total_pr_summaries_created: Some(20),
                    is_custom_model: false,
                    custom_model_training_date: None,
                    languages: None,
                }]),
            }]),
        }),
        copilot_dotcom_chat: Some(CopilotDotcomChat {
            total_engaged_users: 10,
            models: Some(vec![Model {
                name: "gpt-4".to_string(),
                total_engaged_users: 8,
                total_chats: Some(50),
                total_chat_copy_events: None,
                total_chat_insertion_events: None,
                total_pr_summaries_created: None,
                is_custom_model: false,
                custom_model_training_date: None,
                languages: None,
            }]),
        }),
        copilot_dotcom_pull_requests: None,
    }])
}

/// Create test metrics with specified active and engaged users
pub fn create_test_metrics_with_params(active_users: i64, engaged_users: i64) -> CopilotMetrics {
    let mut metrics = create_test_metrics();
    metrics.total_active_users = Some(active_users);
    metrics.total_engaged_users = Some(engaged_users);
    metrics
}

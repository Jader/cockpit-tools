// Codex 账号测试：切号通知的使用建议排序。
// 测试与生产实现共享 super 作用域，验证重置时间和额度窗口规则。
    fn make_usage_recommendation_account(
        id: &str,
        email: &str,
        hourly_percentage: i32,
        weekly_percentage: i32,
        weekly_reset_time: Option<i64>,
        last_used: i64,
    ) -> CodexAccount {
        let mut account = CodexAccount::new(
            id.to_string(),
            email.to_string(),
            CodexTokens {
                id_token: "id-token".to_string(),
                access_token: "access-token".to_string(),
                refresh_token: None,
            },
        );
        account.quota = Some(CodexQuota {
            hourly_percentage,
            hourly_reset_time: None,
            hourly_window_minutes: Some(300),
            hourly_window_present: Some(true),
            weekly_percentage,
            weekly_reset_time,
            weekly_window_minutes: Some(10_080),
            weekly_window_present: Some(true),
            raw_data: None,
        });
        account.last_used = last_used;
        account
    }

    #[test]
    fn usage_recommendation_prefers_earliest_weekly_reset_without_fixed_window() {
        let now = 1_700_000_000;
        let later = make_usage_recommendation_account(
            "later",
            "later@example.com",
            99,
            99,
            Some(now + 30 * 24 * 60 * 60),
            1,
        );
        let earlier = make_usage_recommendation_account(
            "earlier",
            "earlier@example.com",
            20,
            5,
            Some(now + 7 * 24 * 60 * 60),
            2,
        );

        let picked = pick_best_usage_recommendation(
            vec![
                build_usage_recommendation_candidate(&later, now).expect("later candidate"),
                build_usage_recommendation_candidate(&earlier, now).expect("earlier candidate"),
            ],
            now,
        )
        .expect("recommendation");

        assert_eq!(picked.account_id, "earlier");
        assert_eq!(picked.account_label, "earlier@example.com");
        assert!(picked.reason.contains("Weekly 将在 7 天后重置"));
        assert!(picked.reason.contains("Weekly 5%"));
        assert!(picked.reason.contains("5h 20%"));
    }

    #[test]
    fn usage_recommendation_requires_hourly_and_weekly_quota() {
        let now = 1_700_000_000;
        let no_hourly = make_usage_recommendation_account(
            "no-hourly",
            "no-hourly@example.com",
            0,
            50,
            Some(now + 60 * 60),
            1,
        );
        let no_weekly = make_usage_recommendation_account(
            "no-weekly",
            "no-weekly@example.com",
            50,
            0,
            Some(now + 60 * 60),
            2,
        );

        assert!(build_usage_recommendation_candidate(&no_hourly, now).is_none());
        assert!(build_usage_recommendation_candidate(&no_weekly, now).is_none());
    }

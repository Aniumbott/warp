use super::*;
use crate::cloud_environment::CodeForge;

#[test]
fn additional_source_repos_round_trip_and_is_optional() {
    let snapshot = AgentConfigSnapshot {
        repositories: RepositoryConfigSnapshot {
            additional_source_repos: Some(vec![SourceRepo::new(
                CodeForge::GitHub,
                "warpdotdev".to_string(),
                "warp".to_string(),
            )]),
            ..Default::default()
        },
        ..Default::default()
    };
    let json = serde_json::to_value(&snapshot).unwrap();
    assert_eq!(
        json["additional_source_repos"][0],
        serde_json::json!({
            "code_forge": "GITHUB",
            "owner": "warpdotdev",
            "repo": "warp"
        })
    );

    let decoded: AgentConfigSnapshot = serde_json::from_value(json).unwrap();
    assert_eq!(decoded, snapshot);
    assert_eq!(
        decoded.repositories.into_sources(),
        RunRepositorySources::Legacy {
            additional: vec![SourceRepo::new(
                CodeForge::GitHub,
                "warpdotdev".to_string(),
                "warp".to_string(),
            )]
        }
    );

    let legacy: AgentConfigSnapshot = serde_json::from_value(serde_json::json!({})).unwrap();
    assert!(legacy.is_empty());
    assert_eq!(
        legacy.repositories.into_sources(),
        RunRepositorySources::Legacy { additional: vec![] }
    );
}

#[test]
fn empty_factory_plan_preserves_webhook_sources_without_cloning_them() {
    let raw = serde_json::json!({
        "additional_source_repos": [{"code_forge": "GITHUB", "owner": "acme", "repo": "webhook"}],
        "source_repos_to_clone": [],
        "deferred_source_repos": [{"code_forge": "GITLAB", "owner": "platform/backend", "repo": "api"}]
    });
    let snapshot: AgentConfigSnapshot = serde_json::from_value(raw.clone()).unwrap();

    assert_eq!(serde_json::to_value(&snapshot).unwrap(), raw);
    assert_eq!(
        snapshot.repositories.into_sources(),
        RunRepositorySources::FactoryPlan {
            eager: vec![],
            deferred: vec![SourceRepo::new(
                CodeForge::GitLab,
                "platform/backend".to_string(),
                "api".to_string(),
            )],
        }
    );
}

#[test]
fn null_plan_retains_legacy_sources_and_ignores_orphan_deferred_repos() {
    let snapshot: AgentConfigSnapshot = serde_json::from_value(serde_json::json!({
        "additional_source_repos": [{"code_forge": "GITHUB", "owner": "acme", "repo": "webhook"}],
        "source_repos_to_clone": null,
        "deferred_source_repos": [{"code_forge": "GITLAB", "owner": "platform/backend", "repo": "api"}]
    }))
    .unwrap();

    assert_eq!(
        serde_json::to_value(&snapshot).unwrap(),
        serde_json::json!({
            "additional_source_repos": [{"code_forge": "GITHUB", "owner": "acme", "repo": "webhook"}],
            "deferred_source_repos": [{"code_forge": "GITLAB", "owner": "platform/backend", "repo": "api"}]
        })
    );
    let sources = snapshot.repositories.into_sources();
    assert!(sources.deferred_repos().is_empty());
    assert_eq!(
        sources,
        RunRepositorySources::Legacy {
            additional: vec![SourceRepo::new(
                CodeForge::GitHub,
                "acme".to_string(),
                "webhook".to_string(),
            )],
        }
    );
}

#[test]
fn additional_source_repos_make_snapshot_non_empty() {
    let snapshot = AgentConfigSnapshot {
        repositories: RepositoryConfigSnapshot {
            additional_source_repos: Some(vec![SourceRepo::new(
                CodeForge::GitHub,
                "warpdotdev".to_string(),
                "warp".to_string(),
            )]),
            ..Default::default()
        },
        ..Default::default()
    };
    assert!(!snapshot.is_empty());
}

#[test]
fn source_repos_to_clone_absent_empty_and_populated_round_trip() {
    let absent: AgentConfigSnapshot = serde_json::from_value(serde_json::json!({})).unwrap();
    assert!(absent.is_empty());
    assert_eq!(serde_json::to_value(absent).unwrap(), serde_json::json!({}));

    let empty = AgentConfigSnapshot {
        repositories: RepositoryConfigSnapshot {
            source_repos_to_clone: Some(vec![]),
            ..Default::default()
        },
        ..Default::default()
    };
    let json = serde_json::to_value(&empty).unwrap();
    assert_eq!(json["source_repos_to_clone"], serde_json::json!([]));
    let decoded: AgentConfigSnapshot = serde_json::from_value(json).unwrap();
    assert_eq!(decoded, empty);
    // `Some(empty)` is authoritative, not the same thing as an absent field.
    assert!(!empty.is_empty());
    assert_eq!(
        decoded.repositories.into_sources(),
        RunRepositorySources::FactoryPlan {
            eager: vec![],
            deferred: vec![],
        }
    );

    let populated = AgentConfigSnapshot {
        repositories: RepositoryConfigSnapshot {
            source_repos_to_clone: Some(vec![SourceRepo::new(
                CodeForge::GitHub,
                "warpdotdev".to_string(),
                "warp".to_string(),
            )]),
            deferred_source_repos: vec![SourceRepo::new(
                CodeForge::GitLab,
                "platform/backend".to_string(),
                "api".to_string(),
            )],
            ..Default::default()
        },
        ..Default::default()
    };
    let json = serde_json::to_value(&populated).unwrap();
    assert_eq!(
        json["source_repos_to_clone"][0],
        serde_json::json!({"code_forge": "GITHUB", "owner": "warpdotdev", "repo": "warp"})
    );
    assert_eq!(
        json["deferred_source_repos"][0],
        serde_json::json!({"code_forge": "GITLAB", "owner": "platform/backend", "repo": "api"})
    );
    let decoded: AgentConfigSnapshot = serde_json::from_value(json).unwrap();
    assert_eq!(decoded, populated);
    assert_eq!(
        decoded.repositories.into_sources(),
        RunRepositorySources::FactoryPlan {
            eager: vec![SourceRepo::new(
                CodeForge::GitHub,
                "warpdotdev".to_string(),
                "warp".to_string(),
            )],
            deferred: vec![SourceRepo::new(
                CodeForge::GitLab,
                "platform/backend".to_string(),
                "api".to_string(),
            )],
        }
    );
}

#[test]
fn experimental_values_remain_uninterpreted_across_versions() {
    let raw = serde_json::json!({
        "experimental": {
            "identityOnlySystemPrompt": true,
            "future": null,
            "nestedFromNewerServer": {"value": [1, false]}
        }
    });
    let snapshot: AgentConfigSnapshot = serde_json::from_value(raw.clone()).unwrap();
    assert!(!snapshot.is_empty());
    assert_eq!(serde_json::to_value(snapshot).unwrap(), raw);
    assert!(
        serde_json::from_value::<AgentConfigSnapshot>(serde_json::json!({}))
            .unwrap()
            .experimental
            .is_none()
    );
}

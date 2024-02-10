use crate::config;
/// Creates a list of tags.
pub async fn get_tags(env: &Option<String>, tag: &config::Tag) -> Vec<(String, bool)> {
    let mut tags: Vec<(String, bool)> = vec![];
    match tag {
        config::Tag::Commit(commit) => match &env {
            Option::Some(name) => tags.push((format!("{name}-{commit}"), false)),
            Option::None => (),
        },
        config::Tag::SemVer(version) => {
            match &env {
                Option::Some(name) => tags.push((format!("{name}-{}", version), false)),
                Option::None => (),
            }

            tags.push((format!("{}.{}", version.major, version.minor), true));
            tags.push((format!("{}", version.major), true));
        }
    }
    match &env {
        Option::Some(name) => tags.push((format!("{name}-active"), true)),
        Option::None => (),
    }
    tags.push(("latest".to_owned(), true));
    tags
}

#[cfg(test)]
mod tests {
    use super::*;
    use config::Config;
    use std::env;
    #[tokio::test]
    async fn test_get_tags_commit() {
        let configs = vec![
            Config::create(
                Ok(String::from("test")),
                Ok(String::from("foo")),
                Err(env::VarError::NotPresent),
                Err(env::VarError::NotPresent),
                Ok(String::from("6c7faa17e3d6e3d6e356dd9e0d0f1dc2f3b4b174")),
            )
            .unwrap(),
            Config::create(
                Ok(String::from("test")),
                Ok(String::from("foo")),
                Err(env::VarError::NotPresent),
                Ok(String::from("")),
                Ok(String::from("6c7faa17e3d6e3d6e356dd9e0d0f1dc2f3b4b174")),
            )
            .unwrap(),
        ];
        for config in configs {
            assert_eq!(
                get_tags(&config.environment, &config.tag).await,
                vec![
                    (
                        "test-6c7faa17e3d6e3d6e356dd9e0d0f1dc2f3b4b174".to_owned(),
                        false
                    ),
                    ("test-active".to_owned(), true),
                    ("latest".to_owned(), true)
                ]
            );
        }
    }

    #[tokio::test]
    async fn test_get_tags_version() {
        let configs = vec![
            Config::create(
                Ok(String::from("test")),
                Ok(String::from("foo")),
                Err(env::VarError::NotPresent),
                Ok(String::from("1.2.3")),
                Err(env::VarError::NotPresent),
            )
            .unwrap(),
            Config::create(
                Ok(String::from("test")),
                Ok(String::from("foo")),
                Err(env::VarError::NotPresent),
                Ok(String::from("1.2.3")),
                Ok(String::from("")),
            )
            .unwrap(),
        ];
        for config in configs {
            assert_eq!(
                get_tags(&config.environment, &config.tag).await,
                vec![
                    ("test-1.2.3".to_owned(), false),
                    ("1.2".to_owned(), true),
                    ("1".to_owned(), true),
                    ("test-active".to_owned(), true),
                    ("latest".to_owned(), true)
                ]
            );
        }
    }

    #[tokio::test]
    async fn test_get_tags_commit_no_env() {
        let configs = vec![
            Config::create(
                Err(env::VarError::NotPresent),
                Ok(String::from("foo")),
                Err(env::VarError::NotPresent),
                Err(env::VarError::NotPresent),
                Ok(String::from("6c7faa17e3d6e3d6e356dd9e0d0f1dc2f3b4b174")),
            )
            .unwrap(),
            Config::create(
                Ok(String::from("")),
                Ok(String::from("foo")),
                Err(env::VarError::NotPresent),
                Err(env::VarError::NotPresent),
                Ok(String::from("6c7faa17e3d6e3d6e356dd9e0d0f1dc2f3b4b174")),
            )
            .unwrap(),
        ];
        for config in configs {
            assert_eq!(
                get_tags(&config.environment, &config.tag).await,
                vec![("latest".to_owned(), true)]
            );
        }
    }

    #[tokio::test]
    async fn test_get_tags_version_no_env() {
        let configs = vec![
            Config::create(
                Err(env::VarError::NotPresent),
                Ok(String::from("foo")),
                Err(env::VarError::NotPresent),
                Ok(String::from("1.2.3")),
                Err(env::VarError::NotPresent),
            )
            .unwrap(),
            Config::create(
                Ok(String::from("")),
                Ok(String::from("foo")),
                Err(env::VarError::NotPresent),
                Ok(String::from("1.2.3")),
                Err(env::VarError::NotPresent),
            )
            .unwrap(),
        ];
        for config in configs {
            assert_eq!(
                get_tags(&config.environment, &config.tag).await,
                vec![
                    ("1.2".to_owned(), true),
                    ("1".to_owned(), true),
                    ("latest".to_owned(), true)
                ]
            );
        }
    }

    #[tokio::test]
    async fn test_get_tags_app_underscore_name() {
        let configs = vec![Config::create(
            Ok(String::from("test")),
            Ok(String::from("idx_geo_search")),
            Err(env::VarError::NotPresent),
            Ok(String::from("6c7faa17e3d6e3d6e356dd9e0d0f1dc2f3b4b174")),
            Err(env::VarError::NotPresent),
        )
        .unwrap()];
        for config in configs {
            assert_eq!(
                get_tags(&config.environment, &config.tag).await,
                vec![
                    (
                        "test-6c7faa17e3d6e3d6e356dd9e0d0f1dc2f3b4b174".to_owned(),
                        false
                    ),
                    ("test-active".to_owned(), true),
                    ("latest".to_owned(), true)
                ]
            );
        }
    }
}

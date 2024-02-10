use color_eyre::{eyre::eyre, Result};
use semver::Version;
use std::{
    env::{self, VarError},
    str::FromStr,
};

#[derive(PartialEq, Debug)]
pub struct Config {
    pub application: String,
    pub environment: Option<String>,
    pub tag: Tag,
}

#[derive(PartialEq, Debug)]
pub enum Tag {
    Commit(String),
    SemVer(Version),
}

impl Config {
    pub fn new() -> Result<Self> {
        Config::create(
            env::var("ENVIRONMENT"),
            env::var("APPLICATION"),
            env::var("IMAGE_NAME"),
            env::var("IMAGE_TAG"),
            env::var("COMMIT"),
        )
    }

    pub fn create(
        environment: Result<String, VarError>,
        application: Result<String, VarError>,
        image_name: Result<String, VarError>,
        image_tag: Result<String, VarError>,
        commit: Result<String, VarError>,
    ) -> Result<Self> {
        let application = if image_name.as_ref().is_ok_and(|name| !name.is_empty()) {
            image_name?
        } else {
            application?.replace('_', "-")
        };
        let environment = if environment.as_ref().is_ok_and(|env| !env.is_empty()) {
            environment.ok()
        } else {
            None
        };
        let tag = if image_tag.as_ref().is_ok_and(|tag| !tag.is_empty()) {
            Tag::from_str(&image_tag?).map_err(|_| eyre!("Unable to parse tag"))?
        } else {
            Tag::from_str(&commit?).map_err(|_| eyre!("Unable to parse tag"))?
        };

        Ok(Self {
            application,
            environment,
            tag,
        })
    }
}

impl FromStr for Tag {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, ()> {
        if s.len() == 40 && s.chars().all(|c| c.is_ascii_hexdigit()) {
            return Ok(Self::Commit(s.to_owned()));
        }
        let version = Version::from_str(s.trim_start_matches("release-"));
        match version {
            Err(_) => Err(()),
            Ok(v) => Ok(Tag::SemVer(v)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_config() {
        let tests = vec![
            (
                Config::create(
                    Ok(String::from("test")),
                    Ok(String::from("foo")),
                    Err(env::VarError::NotPresent),
                    Err(env::VarError::NotPresent),
                    Ok(String::from("6c7faa17e3d6e3d6e356dd9e0d0f1dc2f3b4b174")),
                )
                .unwrap(),
                Config {
                    environment: Some(String::from("test")),
                    application: String::from("foo"),
                    tag: Tag::Commit(String::from("6c7faa17e3d6e3d6e356dd9e0d0f1dc2f3b4b174")),
                },
            ),
            (
                Config::create(
                    Ok(String::from("test")),
                    Ok(String::from("foo")),
                    Ok(String::from("bar")), // Application overriden with image_name
                    Err(env::VarError::NotPresent),
                    Ok(String::from("6c7faa17e3d6e3d6e356dd9e0d0f1dc2f3b4b174")),
                )
                .unwrap(),
                Config {
                    environment: Some(String::from("test")),
                    application: String::from("bar"),
                    tag: Tag::Commit(String::from("6c7faa17e3d6e3d6e356dd9e0d0f1dc2f3b4b174")),
                },
            ),
            (
                Config::create(
                    Ok(String::from("test")),
                    Ok(String::from("idx_geo_search")),
                    Ok(String::from("")), // Application overriden with image_name
                    Err(env::VarError::NotPresent),
                    Ok(String::from("6c7faa17e3d6e3d6e356dd9e0d0f1dc2f3b4b174")),
                )
                .unwrap(),
                Config {
                    environment: Some(String::from("test")),
                    application: String::from("idx-geo-search"),
                    tag: Tag::Commit(String::from("6c7faa17e3d6e3d6e356dd9e0d0f1dc2f3b4b174")),
                },
            ),
        ];

        for test in tests {
            assert_eq!(test.0, test.1);
        }
    }
}

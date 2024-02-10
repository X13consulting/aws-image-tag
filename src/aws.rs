use color_eyre::{eyre::eyre, Result};

use tokio::task::JoinSet;
use tracing::{info, warn};

use crate::config;

/// Function receives a ECR manifest and adds tags to it.
/// Arguments:
/// tags: Vec of tuples. First element of tuple contains the image tag
///       If the second element is true, existing image tags with that
///       name will be deleted before it is added to this manifest.
pub async fn tag_image(
    client: &aws_sdk_ecr::Client,
    repository: &str,
    manifest: &str,
    tags: Vec<(String, bool)>,
) -> Result<()> {
    let mut rsps = JoinSet::new();
    for tag in &tags {
        if tag.1 {
            let image_ids = aws_sdk_ecr::types::ImageIdentifier::builder()
                .set_image_tag(Some(tag.0.to_owned()))
                .build();
            rsps.spawn(
                client
                    .batch_delete_image()
                    .repository_name(repository)
                    .image_ids(image_ids)
                    .send(),
            );
        }
    }
    while let Some(rsp) = rsps.join_next().await {
        for image_ids in rsp??.image_ids().into_iter() {
            for image_id in image_ids.iter() {
                let image_tag = image_id.image_tag().ok_or(eyre!("foo"))?;
                let image_digest = image_id.image_digest().ok_or(eyre!("foo"))?;
                info!(repository, image_tag, image_digest, "Deleted image tag");
            }
        }
    }
    let mut rsps = JoinSet::new();
    for tag in &tags {
        rsps.spawn(
            client
                .put_image()
                .repository_name(repository)
                .image_manifest(manifest)
                .image_tag(&tag.0)
                .send(),
        );
    }
    while let Some(rsp) = rsps.join_next().await {
        let response = rsp?.map_err(aws_sdk_ecr::Error::from);
        if let Err(aws_sdk_ecr::Error::ImageAlreadyExistsException(ex)) = response {
            warn!(repository, "{ex}");
        } else {
            for image in response?.image.into_iter() {
                let image_id = image
                    .image_id()
                    .ok_or(eyre!("failed getting image id for image"))?;
                let image_tag = image_id
                    .image_tag()
                    .ok_or(eyre!("failed getting image tag for image"))?;
                let image_digest = image_id
                    .image_digest()
                    .ok_or(eyre!("failed getting image tag for image"))?;
                info!(repository, image_tag, image_digest, "Added image tag");
            }
        }
    }
    Ok(())
}

pub async fn get_manifest(
    client: &aws_sdk_ecr::Client,
    repository: &str,
    tag: &config::Tag,
) -> Result<String> {
    let tag_string = match tag {
        config::Tag::Commit(commit) => commit.clone(),
        config::Tag::SemVer(version) => version.to_string(),
    };
    let rsp = client
        .batch_get_image()
        .repository_name(repository)
        .image_ids(
            aws_sdk_ecr::types::ImageIdentifier::builder()
                .set_image_tag(Some(tag_string))
                .build(),
        )
        .send()
        .await?;
    if let Some(image) = rsp.images() {
        let manifest: String = image
            .iter()
            .next()
            .ok_or(eyre!(
                "empty image list. check application build and image name. image might have been deleted by AWS ECR lifecycle rules."
            ))?
            .image_manifest()
            .ok_or(eyre!("invalid image manifest"))?
            .to_owned();
        return Ok(manifest);
    }
    Err(eyre!("No manifest"))
}

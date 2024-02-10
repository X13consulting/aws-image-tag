Opinionated application to be used in CI builds to tag container images in AWS ECR.

It is used in conjunction with lifecycle rules in AWS ECR to keep n-number of
images that has been promoted to various environments.

The application expects these environment variables:
AWS_ACCESS_KEY_ID
AWS_SECRET_ACCESS_KEY
APPLICATION
ENVIRONMENT
IMAGE_TAG
COMMIT



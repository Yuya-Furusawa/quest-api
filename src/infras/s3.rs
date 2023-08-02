use aws_sdk_s3::Client;

pub struct S3 {
  client: Client,
}

impl S3 {
  pub fn new(client: Client) -> Self {
    Self { client }
  }
}

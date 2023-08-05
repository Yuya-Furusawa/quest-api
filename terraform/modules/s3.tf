resource "aws_s3_bucket" "images" {
  bucket = "quest-app-images-bucket"
}

resource "aws_s3_bucket_ownership_controls" "images_ownership_controls" {
  bucket = aws_s3_bucket.images.id

  rule {
    object_ownership = "BucketOwnerPreferred"
  }
}

resource "aws_s3_bucket_public_access_block" "images_bucket_access" {
  bucket = aws_s3_bucket.images.id

  block_public_acls       = false
  block_public_policy     = false
  ignore_public_acls      = false
  restrict_public_buckets = false
}

resource "aws_s3_bucket_acl" "images_bucket_acl" {
  depends_on = [
    aws_s3_bucket_ownership_controls.images_ownership_controls,
    aws_s3_bucket_public_access_block.images_bucket_access,
  ]

  bucket = aws_s3_bucket.images.id
  acl    = "public-read"
}

resource "aws_s3_bucket" "images" {
  bucket = "quest-app-images-bucket"
}

resource "aws_s3_bucket_ownership_controls" "images_ownership_controls" {
  bucket = aws_s3_bucket.images.id

  rule {
    object_ownership = "BucketOwnerPreferred"
  }
}

resource "aws_s3_bucket_acl" "example" {
  depends_on = [ aws_s3_bucket_ownership_controls.images_ownership_controls ]

  bucket = aws_s3_bucket.images.id
  acl    = "private"
}

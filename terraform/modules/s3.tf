resource "aws_s3_bucket" "challenge_images" {
  bucket = "challenge-images-bucket"

  tags = {
    Name = "Challenge Images"
  }
}

resource "aws_s3_bucket_policy" "challenge_images_bucket_policy" {
  bucket = aws_s3_bucket.challenge_images.id
  policy = data.aws_iam_policy_document.allow_access.json
}

data "aws_iam_policy_document" "allow_access" {
  statement {
    sid       = "AddPerm"
    effect    = "Allow"
    actions   = ["s3:GetObject"]
    resources = ["${aws_s3_bucket.challenge_images.arn}/*"]

    principals {
      type        = "*"
      identifiers = ["*"]
    }
  }
}

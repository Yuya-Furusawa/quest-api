terraform {
  backend "local" {}

  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "5.4.0"
    }
  }

  required_version = ">= 1.5.0"
}

# provider
provider "aws" {
  region = "ap-northeast-1"

  access_key = "mock_access_key"
  secret_key = "mock_secret_key"

  s3_use_path_style           = true
  skip_credentials_validation = true
  skip_metadata_api_check     = true
  skip_requesting_account_id  = true

  endpoints {
    dynamodb = "http://localstack:4566"
    s3       = "http://localstack:4566"
  }
}

module "resources" {
  source = "../modules"
}

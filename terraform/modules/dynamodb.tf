resource "aws_dynamodb_table" "quests" {
  name           = "quests"
  billing_mode   = "PAY_PER_REQUEST"
  hash_key       = "QuestId"

  attribute {
    name = "QuestId"
    type = "S"
  }
}

resource "aws_dynamodb_table" "users" {
  name           = "users"
  billing_mode   = "PAY_PER_REQUEST"
  hash_key       = "UserId"

  attribute {
    name = "UserId"
    type = "S"
  }

  attribute {
    name = "UserEmail"
    type = "S"
  }

  global_secondary_index {
    name               = "UserEmailIndex"
    hash_key           = "UserEmail"
    projection_type    = "ALL"
  }
}

resource "aws_dynamodb_table" "user_participating_quests" {
  name           = "user_participating_quests"
  billing_mode   = "PAY_PER_REQUEST"
  hash_key       = "UserId"
  range_key      = "QuestId"

  attribute {
    name = "UserId"
    type = "S"
  }

  attribute {
    name = "QuestId"
    type = "S"
  }

  global_secondary_index {
    name               = "QuestUserIndex"
    hash_key           = "QuestId"
    range_key          = "UserId"
    projection_type    = "ALL"
  }
}

resource "aws_dynamodb_table" "challenges" {
  name          = "challenges"
  billing_mode  = "PAY_PER_REQUEST"
  hash_key      = "QuestId"
  range_key     = "ChallengeId" # QuestローカルなChallengeId

  attribute {
    name = "ChallengeId"
    type = "S"
  }

  attribute {
    name = "QuestId"
    type = "S"
  }
}

resource " aws_dynamodb_table" "user_completed_challenges" {
  name         = "user_completed_challenges"
  billing_mode = "PAY_PER_REQUEST"
  hash_key     = "UserId"
  range_key    = "ChallengeId"

  attribute {
    name = "UserId"
    type = "S"
  }

  attribute {
    name = "ChallengeId"
    type = "S"
  }

  global_secondary_index {
    name               = "ChallengeUserIndex"
    hash_key           = "ChallengeId"
    range_key          = "UserId"
    projection_type    = "ALL"
  }
}

-- user_questsからuser_paricipating_questsにRename
ALTER TABLE user_quests RENAME TO user_participating_quests;

-- idを削除する（use_idとquest_idのペアが実質的にidとなるため）
ALTER TABLE user_participating_quests DROP COLUMN id;

-- (user_id, quest_id)のペアに一意制約を設ける
ALTER TABLE user_participating_quests
ADD CONSTRAINT unique_user_quest_pair UNIQUE (user_id, quest_id);


-- user_challengesからuser_completed_challengesにRename
ALTER TABLE user_challenges RENAME TO user_completed_challenges;

-- idを削除する（use_idとchallenge_idのペアが実質的にidとなるため）
ALTER TABLE user_completed_challenges DROP COLUMN id;

-- (user_id, challenge_id)のペアに一意制約を設ける
ALTER TABLE user_completed_challenges
ADD CONSTRAINT unique_user_challenge_pair UNIQUE (user_id, challenge_id);

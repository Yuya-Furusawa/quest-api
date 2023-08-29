#!/bin/bash
set -e

####################
## base branchに対して、fast-forward merge可能かどうかをチェックする
####################

git fetch origin $GITHUB_BASE_REF

# BASE_COMMITを取得する
MERGE_BASE_COMMIT=$(git merge-base origin/$GITHUB_BASE_REF HEAD)

# base branchの最新コミットを取得する
BASE_HEAD_COMMIT=$(git rev-parse origin/$GITHUB_BASE_REF)

# Fast Forwardの可否をチェック
if [ "$MERGE_BASE_COMMIT" = "$BASE_HEAD_COMMIT" ]; then
  echo "Fast-forward mergeable."
else
  echo "Not fast-forward mergeable."
  exit 1
fi

####################
## BASE_COMMITからHEADまでの履歴が一直接かどうかをチェックする
####################

MERGE_COMMITS=$(git rev-list --merges $MERGE_BASE_COMMIT..HEAD)

if [ -z "$MERGE_COMMITS" ]; then
  echo "Merge commits not found."
else
  echo "Merge commits found. : ${MERGE_COMMITS}"
  exit 1
fi

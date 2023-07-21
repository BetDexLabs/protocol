#!/bin/bash

BRANCH_NAME=$1
if [ -z "$1" ]; then
  read -p "Branch name to sync changes for: " BRANCH_NAME
fi;

git checkout ${BRANCH_NAME}
git remote add protocol git@github.com:BetDexLabs/protocol.git
git push protocol ${BRANCH_NAME}
git checkout develop-ci

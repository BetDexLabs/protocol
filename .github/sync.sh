#!/bin/bash

read -p "Branch name to sync changes for: " BRANCH_NAME
PRIVATE_FORK_PATH="$(pwd)"
PUBLIC_FORK_PATH="$(pwd)/../protocol"
FILES_MODIFIED_PATH="${PRIVATE_FORK_PATH}/files-modified"

# track any changed files, ignoring changes to .github/ ci files
git checkout ${BRANCH_NAME} && git diff --name-only origin/main... -- . ':!:.github/*' | cat > ${FILES_MODIFIED_PATH}

# switch to public fork
cd $PUBLIC_FORK_PATH
git checkout main && git pull origin main

# if branch already exists checkout, else create
if [ `git rev-parse --verify ${BRANCH_NAME} 2>/dev/null` ]
then
   echo "Checked out existing branch ${BRANCH_NAME} at ${pwd}"
   git checkout ${BRANCH_NAME}
else
   echo "Creating new branch ${BRANCH_NAME} at ${pwd}"
   git checkout -b ${BRANCH_NAME}
fi

# sync files from -private repo to public fork
rsync -av --files-from=${FILES_MODIFIED_PATH} ${PRIVATE_FORK_PATH} ${PUBLIC_FORK_PATH} --relative
rm ${FILES_MODIFIED_PATH}

# create new commit, prompt for message, and push
git add .
read -p "Please enter commit message for your changes: " COMMIT_MESSAGE
git commit -m "${COMMIT_MESSAGE}"
git push origin ${BRANCH_NAME}

cd $PRIVATE_FORK_PATH && git checkout develop

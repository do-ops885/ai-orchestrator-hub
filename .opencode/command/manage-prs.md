---
title: "Manage Pull Requests"
description: "Automate GitHub PR management including review, merge testing, and handling"
agent: "github-pr-manager"
command: "/manage-prs"
---

# Manage Pull Requests

This command automates the management of GitHub pull requests using the github-pr-manager agent.

## Prompt for Agent

Process all open pull requests in the current repository. For each pull request:

1. **Review Changes**: Analyze the code changes for quality, adherence to coding standards, and potential issues.

2. **Check Conflicts**: Verify if there are any merge conflicts with the base branch.

3. **Test Merge**: Attempt to merge the PR into a test branch to ensure it doesn't break the build or introduce regressions.

4. **Run Tests**: If automated tests are configured, run them against the merged code.

5. **Handle Appropriately**:
   - If all checks pass and the PR meets quality standards, merge it.
   - If there are issues, add comments requesting changes.
   - If the PR is outdated or no longer relevant, close it with an appropriate message.

6. **Update Status**: Provide a summary of actions taken for each PR.

Ensure all actions are performed safely and follow the project's contribution guidelines.

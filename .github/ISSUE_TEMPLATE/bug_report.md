---
name: Bug Report
description: Report a bug or unexpected behavior
title: "[BUG] "
labels: ["bug", "triage"]
assignees: []
body:
  - type: textarea
    id: description
    attributes:
      label: Description
      description: A clear and concise description of the bug
      placeholder: What happened?
    validations:
      required: true
  - type: textarea
    id: reproduction
    attributes:
      label: Steps to Reproduce
      description: Steps to reproduce the behavior
      placeholder: |
        1. Go to '...'
        2. Click on '...'
        3. See error
    validations:
      required: true
  - type: textarea
    id: expected
    attributes:
      label: Expected Behavior
      description: What you expected to happen
    validations:
      required: true
  - type: dropdown
    id: component
    attributes:
      label: Component
      description: Which component is affected?
      options:
        - Backend (Rust)
        - Frontend (TypeScript/React)
        - Infrastructure
        - Documentation
        - Other
    validations:
      required: true
  - type: input
    id: version
    attributes:
      label: Version
      description: Version of the software
      placeholder: e.g., v1.0.0
  - type: textarea
    id: additional
    attributes:
      label: Additional Context
      description: Add any other context about the problem here
      placeholder: Screenshots, logs, etc.
---

**Environment:**
- OS: [e.g., Linux, macOS, Windows]
- Browser: [e.g., Chrome 91, Firefox 89]
- Rust version: [e.g., 1.58.0]
- Node.js version: [e.g., 16.14.0]

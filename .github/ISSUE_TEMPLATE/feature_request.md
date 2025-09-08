---
name: Feature Request
description: Suggest a new feature or enhancement
title: "[FEATURE] "
labels: ["enhancement", "triage"]
assignees: []
body:
  - type: textarea
    id: summary
    attributes:
      label: Summary
      description: A brief summary of the feature request
    validations:
      required: true
  - type: textarea
    id: problem
    attributes:
      label: Problem/Use Case
      description: What problem does this feature solve? What is the use case?
    validations:
      required: true
  - type: textarea
    id: solution
    attributes:
      label: Proposed Solution
      description: Describe the solution you'd like
    validations:
      required: true
  - type: textarea
    id: alternatives
    attributes:
      label: Alternative Solutions
      description: Describe any alternative solutions or features you've considered
  - type: dropdown
    id: component
    attributes:
      label: Component
      description: Which component would this affect?
      options:
        - Backend (Rust)
        - Frontend (TypeScript/React)
        - Infrastructure
        - Documentation
        - Other
    validations:
      required: true
  - type: textarea
    id: additional
    attributes:
      label: Additional Context
      description: Add any other context or screenshots about the feature request here
---

**Priority:** [High/Medium/Low]
**Estimated effort:** [Small/Medium/Large]

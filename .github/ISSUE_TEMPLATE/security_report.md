---
name: Security Report
description: Report a security vulnerability
title: "[SECURITY] "
labels: ["security", "triage"]
assignees: []
body:
  - type: textarea
    id: summary
    attributes:
      label: Summary
      description: A brief summary of the security issue
    validations:
      required: true
  - type: dropdown
    id: severity
    attributes:
      label: Severity
      description: How severe is this security issue?
      options:
        - Critical
        - High
        - Medium
        - Low
        - Info
    validations:
      required: true
  - type: textarea
    id: description
    attributes:
      label: Description
      description: Detailed description of the security vulnerability
    validations:
      required: true
  - type: textarea
    id: impact
    attributes:
      label: Impact
      description: What is the potential impact of this vulnerability?
    validations:
      required: true
  - type: textarea
    id: reproduction
    attributes:
      label: Steps to Reproduce
      description: How can this vulnerability be reproduced?
    validations:
      required: true
  - type: textarea
    id: mitigation
    attributes:
      label: Suggested Mitigation
      description: Any suggested fixes or workarounds
  - type: input
    id: contact
    attributes:
      label: Contact Information
      description: How can we contact you for more details?
      placeholder: Email or other contact method
---

**Note:** Please do not disclose sensitive details in public issues. If this is a critical vulnerability, consider using the security advisory feature instead.
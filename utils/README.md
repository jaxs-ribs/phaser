# /utils

This directory is a collection of shared, cross-cutting utility modules that don't fit into any of the other specific categories.

Potential utilities include:
-   **Budgeting**: A module to track API token usage and cost.
-   **Secret Stripping**: Functions to redact sensitive information from code before it's sent to the LLM.
-   **Git Client**: A wrapper for `git` commands to manage branches and commits.
-   **OCR**: An OCR tool for the visual feedback loop if a non-multimodal LLM is used. 
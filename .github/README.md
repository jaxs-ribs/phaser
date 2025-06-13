# /.github

This directory stores GitHub-specific files, primarily for Continuous Integration and Continuous Deployment (CI/CD).

The `workflows/` subdirectory contains the YAML files that define our GitHub Actions. The initial `ci.yml` workflow will handle linting and testing on every push and pull request to ensure code quality and prevent regressions. 
# /hooks

This directory contains scripts that the orchestrator can "hook" into to interact with the environment. These are the tools that enable the autonomous TDD and observability loops.

Examples of hooks include:
-   `autotest`: Runs the project's test suite (e.g., `pytest`, `npm test`).
-   `screenshot`: Takes a screenshot of a specific window or the entire screen.
-   `video`: Records a short video or GIF of screen activity.

Each hook is a self-contained tool that the orchestrator can invoke to observe the results of a code change. 
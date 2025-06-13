# /orchestrator

This is the brain of the application. The orchestrator is the central controller that manages the entire workflow, from receiving user input to dispatching tasks and producing the final output.

Its responsibilities include:
1.  **Parsing Input**: Receiving the transcribed text from the `/voice` module.
2.  **Building Prompts**: Using the `/index` RAG pipeline and `/prompt` templates to construct a token-efficient prompt.
3.  **Calling the LLM**: Sending the prompt to the selected provider via the `/llm` interface.
4.  **Managing the TDD Loop**: Coordinating with `/hooks` and `/edit` to apply changes, run tests, and iterate until the task is complete.
5.  **Serving the API**: (Later milestones) Hosting the WebSocket server for the mobile companion app. 
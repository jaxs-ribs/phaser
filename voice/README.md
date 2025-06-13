# /voice

This module is responsible for capturing audio input and transcribing it to text.

It will contain:
1.  **Audio Capture**: Code to access the device microphone (`PyAudio` was the initial suggestion, but this can be any native library).
2.  **Transcription**: A wrapper around a speech-to-text engine (like `whisper.cpp`) to convert the captured audio into a text transcript.

This module's output is the primary input for the user's commands to the orchestrator. 
# /mobile

This directory will contain the source code for the mobile companion app (iOS/Android).

The current plan is to use a cross-platform framework like React Native to build the app. The app's primary responsibility is to provide a simple "push-to-talk" interface that records audio and streams it to the backend server via WebSocket.

Development of this component is a parallel track and can happen independently of the core application, as long as it adheres to the API specified in `docs/mobile_api.md`. 
# /demo

This directory contains scripts and resources for demonstrating end-to-end functionality of the system.

A key file here will be the `demo_voice_cli.sh` script, which automates a full vertical-slice test run. For CI/CD purposes, it will use a pre-recorded `.wav` file instead of a live microphone to ensure deterministic and repeatable test runs.

This directory is crucial for integration testing and verifying that all components are working together as expected. 
# Project Agent Instructions

## Overview

This project uses the Codex CLI for AI-assisted development.

## Coding Standards

- Follow PEP 8 for Python code
- Use ESLint for JavaScript
- All functions must have docstrings

## Security

- Never execute arbitrary network requests
- Sanitize all user input
- Use parameterized queries for database operations

## File Access

The agent has access to:
- Source code directories
- Configuration files
- Test fixtures

The agent does NOT have access to:
- .env files
- secrets/ directory
- Production credentials

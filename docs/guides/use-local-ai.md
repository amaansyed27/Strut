# Use Local AI

Strut is designed to work without forcing your design files through a cloud service.

The first local provider target is Ollama.

## Why Use Local AI

- Keep private design files on your machine.
- Work without provider rate limits.
- Test smaller local models for simple edits.
- Use local GPU acceleration when available.

## Intended Flow

1. Install Ollama.
2. Pull a model that supports your workflow.
3. Open Strut Studio.
4. Select Ollama in the provider panel.
5. Run a Plan Mode or edit request.

## Privacy

Local AI does not automatically mean every workflow is private. Strut still needs to show which provider is being used and what context is included before a request runs.

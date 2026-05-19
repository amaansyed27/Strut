# Strut Files

`.strut` is the open file format for Strut animation components.

The format is designed to be:

- portable
- versioned
- inspectable
- validated before playback
- safe to pass between design and engineering workflows

## File Shape

The first format is a ZIP container:

```txt
login-button.strut
  manifest.json
  document.json
  assets/
  previews/
```

## Manifest

`manifest.json` tells the runtime how to read the file.

```json
{
  "format": "strut",
  "schemaVersion": "0.1.0",
  "document": "document.json",
  "createdBy": "strut-studio",
  "minimumRuntime": "0.1.0"
}
```

## Document

`document.json` contains the editable animation model:

- artboards
- nodes
- timelines
- state machines
- inputs
- bindings
- events

## Compatibility

Strut runtimes should reject unsupported major versions and show a clear error instead of trying to play a file incorrectly.

## Validate A File

From the repository root:

```powershell
cargo run -p strut-format --example validate -- samples/login-button.strut
```

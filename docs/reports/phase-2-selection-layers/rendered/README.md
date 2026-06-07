# Phase 2 DOCX Render Fallback

DOCX render command attempted:

```powershell
$py='C:\Users\Amaan\.cache\codex-runtimes\codex-primary-runtime\dependencies\python\python.exe'
$env:PATH='C:\Users\Amaan\.cache\codex-runtimes\codex-primary-runtime\dependencies\bin;' + $env:PATH
& $py 'C:\Users\Amaan\.codex\plugins\cache\openai-primary-runtime\documents\26.601.10930\skills\documents\render_docx.py' 'D:\TheDawnlightGroup\DawnlightLabs\Strut\docs\reports\phase-2-selection-layers\phase-2-selection-layers-report.docx' --output_dir 'D:\TheDawnlightGroup\DawnlightLabs\Strut\docs\reports\phase-2-selection-layers\rendered' --emit_pdf --verbose
```

Exact blocker:

```text
FileNotFoundError: [WinError 2] The system cannot find the file specified
```

The failure occurs while `render_docx.py` tries to launch the LibreOffice/soffice conversion command. No `page-*.png` or PDF render proof was produced.

Fallback visual proof:

- `screenshots/before-phase-1/` contains copied Phase 1 final screenshots for before evidence.
- `screenshots/browser/` contains deterministic localhost Playwright screenshots for Phase 2 after evidence.
- `screenshots/tauri/` contains actual Tauri WebView2 native-shell screenshots for Phase 2 after evidence.

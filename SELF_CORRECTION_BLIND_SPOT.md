# Self-Correction Blind Spot: Why Reflection Was Disabled

## Problem Encountered

After implementing the AI quality feedback loop, **all 8 generated animations were blank**. The preview showed empty or nearly empty canvases despite the generation appearing to succeed.

## Root Cause: The Self-Correction Blind Spot

Research ([arxiv.org/abs/2507.02778](https://arxiv.org/abs/2507.02778)) identifies this as a fundamental limitation of LLM self-correction:

### The Issue
When an LLM reviews its own output, it shares the same:
- **Training biases** - systematic preferences for certain patterns
- **Reasoning blind spots** - inability to detect certain error types
- **Evaluation criteria** - same model judging by its own standards

This creates a **correlated error problem**: the verifier's mistakes are correlated with the generator's mistakes.

### The Failure Modes

**Mode 1: Too Lenient** (original problem)
- Model generates flawed output
- Model reviews and approves it (misses the same errors)
- Result: Poor quality animations ship to users

**Mode 2: Too Strict** (current problem)
- Model generates valid output
- Model applies overly strict criteria during reflection
- Model rejects valid output or over-corrects to blank/broken state
- Result: Blank animations, nothing renders

### Research Findings

- **64.5% failure rate** across 14 models on self-correction tasks
- LLMs can correct errors when presented as **external input**
- LLMs **fail to correct** the same errors in **their own outputs**
- The problem is **structural**, not prompt-engineering fixable

## Why Our Reflection Failed

Our implementation:
```rust
// Generate document
let initial_result = generate(...);

// Ask SAME model to review its OWN output
let reflection_prompt = visual_quality_reflection_prompt(...);
let feedback = call_provider(...); // Same model, same biases

// Replace initial with "improved" version
initial_result = feedback_result; // ← This broke everything
```

The comprehensive validation checklist made it **worse**, not better:
- More rules = more ways to reject valid output
- Stricter criteria = over-correction
- Same model = blind to what actually matters


## Solutions That Actually Work

Based on research, here are proven approaches:

### 1. **Disable Self-Correction** (Immediate Fix - IMPLEMENTED)
```rust
// Commented out the reflection loop
// Return the initial generation result directly
```
**Pros**: Animations work again immediately
**Cons**: Back to original quality issues

### 2. **Context Separation** (Recommended Next Step)
Present the output as if from an external source:
```rust
// Instead of: "Review the document YOU just created"
// Use: "Review this Strut document: [document_json]"
// Don't mention it's the model's own output
```
Research shows this can break the bias correlation.

### 3. **Different Sampling Parameters**
Use different temperature/top_p for generation vs validation:
```rust
// Generation: temperature=0.7 (creative)
// Validation: temperature=0.1 (deterministic/strict)
```
Creates some decorrelation between generator and verifier.

### 4. **External Validator** (Most Robust)
Use a **different validation approach**:
- **Deterministic checks**: Rust validation (already implemented)
- **Structural rules**: JSON schema validation, required fields
- **Visual checks**: Render and check for blank canvas, missing parts
- **Different model**: Use a different AI provider for validation

### 5. **Human-in-the-Loop** (Gold Standard)
- Show generation to user
- Let user request fixes
- User feedback breaks the correlation

## Immediate Action Taken

**Disabled the reflection loop** in `commands.rs`:
- Commented out the entire reflection section
- Added research citation explaining why
- Added TODO for proper external validation

This restores working animations while we implement a better solution.

## Next Steps

1. ✅ **Disable reflection** - animations work again
2. 🔄 **Strengthen upfront constraints** - keep the improved prompts
3. 🔄 **Add deterministic validation** - Rust-side geometry checks
4. 📋 **Consider context separation** - if reflection is re-enabled
5. 📋 **Implement visual validation** - render and check for blank output
6. 📋 **Add user feedback** - let users flag and fix issues

## Key Takeaway

**Self-correction with the same model is fundamentally limited.** The research is clear: you cannot fix this with better prompts. The solution requires **external feedback** - whether from deterministic rules, different models, visual checks, or human input.

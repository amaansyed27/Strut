# AI Quality Feedback Loop Improvements

## Problem Statement

During 3D dice animation generation testing, several quality issues were observed:

1. **Inconsistent styling**: Some dice faces had rounded corners (rx:12) while others had sharp corners (rx:0)
2. **Multi-axis chaos**: Dice spinning and tumbling on multiple axes simultaneously
3. **Face glitches**: Visual artifacts in certain face states (e.g., Face 4)
4. **Poor production quality**: Animations unsuitable for real web/mobile apps

## Solution: Strengthened Reflection Feedback Loop

The AI generation pipeline includes a self-reflection step where the model reviews its own output.
This commit strengthens that feedback mechanism with specific, actionable quality checks.

### Changes Made

#### 1. Enhanced Reflection Prompt (`prompts.rs`)

Created a **comprehensive validation checklist** with 6 mandatory checks:

1. **Corner Radius Consistency** - All rect parts forming same object must use identical rx values
2. **Single-Axis 3D Rotation** - Only ONE rotation axis per timeline (prevent chaotic tumbling)
3. **Complete Geometry** - All required parts must exist (e.g., all 6 dice faces)
4. **Geometry Overlap & Depth** - Overlapping parts need matching/offset dimensions
5. **Animation Smoothness** - Even keyframe distribution with proper easing
6. **Opacity Mutual Exclusion** - State-specific layers start at opacity:0, animate to 1

Each rule includes:
- ✗ **Wrong** examples (what not to do)
- ✓ **Correct** examples (what to do instead)
- → **Action** items (how to fix it)

#### 2. Improved Error Visibility (`commands.rs`)

Changed from silent failure handling to explicit error logging with `eprintln!`:

```rust
// Now logs when reflection parsing fails or API calls fail
match parse_assistant_result_from_text(&feedback_text) {
    Ok(feedback_result) => { /* success */ }
    Err(parse_error) => {
        eprintln!("Quality reflection produced unparseable output: {parse_error}");
    }
}
```


#### 3. Strengthened Upfront Constraints

Updated **initial generation prompt** to prevent issues before they occur:

**GEOMETRY RULES** enhancement:
```
CRITICAL: Apply a CONSISTENT rx value to ALL rect elements that form
the same logical object. If dice body uses rx:12, ALL face borders
and overlays MUST also use rx:12, NOT rx:0 or different values.
```

**ANIMATION QUALITY RULES** enhancement:
```
CRITICAL 3D ROTATION RULE: For 3D spin effects, use ONLY ONE rotation
axis per timeline:
  * "rotation.y" for horizontal spin (coin flip left/right)
  * "rotation.x" for vertical tumble (dice roll forward/backward)  
  * "rotation" for flat 2D spin (no 3D perspective)
  * NEVER mix multiple rotation properties in same timeline
```

### How It Works

The generation flow with reflection:

```
1. User prompt → Initial generation
2. Parse & validate initial document
3. IF generation request:
   a. Serialize document to JSON
   b. Create reflection prompt with validation checklist
   c. Call AI provider again with reflection prompt
   d. Parse improved document
   e. Replace initial result with improved version
4. Return final document
```

### Expected Improvements

With these changes, generated animations should:

✓ Have **consistent styling** (no mixed corner radii)
✓ Use **single-axis rotation** (smooth, realistic 3D motion)
✓ Include **complete geometry** (no missing parts)
✓ Have **smooth animations** (proper keyframe timing and easing)
✓ Work correctly in **production** (web/mobile apps)

### Testing

All existing tests pass (47 passed, 0 failed).

To test with dice generation:
```powershell
npm run studio:dev
# In Studio, try: "Create a smooth, 3D-style rolling dice animation"
```

The reflection step will automatically review and fix quality issues before showing the final result.

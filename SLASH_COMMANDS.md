# Slash Commands: Manual Animation Strategy Selection

## Feature

Added a **slash command menu** in the chat composer that lets you manually select the animation generation strategy.

## Usage

### Open the Menu
Type `/` in the chat box to open the strategy selection menu.

### Available Commands

**`/auto`** (Default)
- Let the AI automatically choose the best style based on your prompt
- Uses keyword detection (same as before)
- **Example**: "Create a dice animation" → AI decides dynamically

**`/svg`** - Simple SVG Vector Style
- Lightweight SVG shapes with CSS animations
- Best for: logos, icons, badges, loaders, buttons, UI elements
- Fast rendering, simple structure
- **Example**: `/svg` then "Create a loading spinner"

**`/sprite`** - Complex Sprite Style
- Layered sprites with semantic rigs
- Best for: mascots, characters, game sprites, expressive animations
- More complex, cinematic feel
- **Example**: `/sprite` then "Create a waving robot"

**`/dynamic`** - Dynamic Provider Plan
- Flexible approach, AI chooses representation
- Best for: 3D effects, complex motion, anything that doesn't fit simple/sprite
- **Example**: `/dynamic` then "Create a 3D rolling dice"

### Visual Indicator

After selecting a strategy, you'll see an indicator in the top-right of the chat box:
- 🎯 **Auto** - Automatic selection
- 🎨 **SVG** - Simple vector style
- 🎮 **Sprite** - Complex sprite style
- ⚡ **Dynamic** - Dynamic plan

### How It Works

1. Type `/` in the chat
2. Menu appears with 4 options
3. Click your choice (or press Escape to close without selecting)
4. The `/` is removed from your prompt
5. Strategy indicator shows your selection
6. When you send the message, the strategy hint is appended to your prompt

### Technical Implementation

The selected strategy adds a hint to your prompt before sending:
- SVG: `[use svg vector style]`
- Sprite: `[use sprite style]`
- Dynamic: `[use dynamic style]`

This guides the AI's generation approach while keeping the prompt readable.

## Benefits

✅ **Manual control** when you know exactly what style you want
✅ **Visual feedback** - see which strategy is active
✅ **Non-intrusive** - auto mode still works as before
✅ **Quick selection** - keyboard-accessible slash commands
✅ **Persistent** - strategy stays selected until you change it

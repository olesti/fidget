---
name: Fidget
description: A quiet, native-feeling tray utility. Neutral surfaces, one blue accent, compact controls.
colors:
  accent-blue: "#2563eb"
  accent-blue-deep: "#1d4ed8"
  live-green: "#22c55e"
  live-green-text: "#16a34a"
  caution-amber: "#d97706"
  error-red: "#ef4444"
  paper: "#ffffff"
  paper-dark: "#171717"
  fill-soft: "#f5f5f5"
  fill-soft-dark: "#262626"
  fill-hover: "#e5e5e5"
  fill-raised-dark: "#404040"
  hairline: "#e5e5e5"
  hairline-strong: "#d4d4d4"
  hairline-dark: "#404040"
  ink: "#171717"
  ink-dark: "#f5f5f5"
  ink-muted: "#737373"
  switch-off: "#d4d4d4"
  switch-off-dark: "#525252"
typography:
  title:
    fontFamily: "system-ui, sans-serif"
    fontSize: "1.25rem"
    fontWeight: 600
    lineHeight: 1.4
  readout:
    fontFamily: "system-ui, sans-serif"
    fontSize: "1.5rem"
    fontWeight: 600
    lineHeight: 1.33
    fontFeature: "tnum"
  body:
    fontFamily: "system-ui, sans-serif"
    fontSize: "14px"
    fontWeight: 400
    lineHeight: 1.4
  label:
    fontFamily: "system-ui, sans-serif"
    fontSize: "0.875rem"
    fontWeight: 500
    lineHeight: 1.43
  caption:
    fontFamily: "system-ui, sans-serif"
    fontSize: "0.75rem"
    fontWeight: 400
    lineHeight: 1.33
rounded:
  sm: "4px"
  md: "8px"
  lg: "12px"
  full: "9999px"
spacing:
  xs: "4px"
  sm: "8px"
  md: "12px"
  lg: "16px"
  xl: "20px"
components:
  button-secondary:
    backgroundColor: "transparent"
    textColor: "{colors.ink}"
    typography: "{typography.label}"
    rounded: "{rounded.md}"
    padding: "8px 12px"
  button-secondary-hover:
    backgroundColor: "{colors.fill-soft}"
  button-primary:
    backgroundColor: "{colors.accent-blue}"
    textColor: "{colors.paper}"
    rounded: "{rounded.sm}"
    padding: "4px 12px"
  button-primary-hover:
    backgroundColor: "{colors.accent-blue-deep}"
  chip:
    backgroundColor: "{colors.fill-soft}"
    textColor: "{colors.ink}"
    typography: "{typography.label}"
    rounded: "{rounded.md}"
    padding: "8px"
  chip-selected:
    backgroundColor: "{colors.accent-blue}"
    textColor: "{colors.paper}"
  card:
    backgroundColor: "{colors.paper}"
    rounded: "{rounded.lg}"
    padding: "12px"
  input:
    backgroundColor: "transparent"
    textColor: "{colors.ink}"
    rounded: "{rounded.sm}"
    padding: "4px 8px"
  switch-track:
    backgroundColor: "{colors.switch-off}"
    rounded: "{rounded.full}"
    height: "24px"
    width: "44px"
  switch-track-on:
    backgroundColor: "{colors.accent-blue}"
---

# Design System: Fidget

## Overview

**Creative North Star: "The Quiet Utility"**

Fidget looks like it shipped with the operating system. Surfaces are neutral, type is the system font at body size, and borders are hairlines. The single blue accent marks what is on, selected, or primary, and nothing else. The app is opened for seconds, so the interface gets out of the way: nothing animates for show, nothing decorates.

The system has two surfaces with the same vocabulary. The tray **Panel** is compact and fixed-size, with a feature switch, a countdown readout, a grid of timer chips and a pause row. The **Settings** window is a centered column (max 36rem) of bordered sections under underline tabs. Both follow the system or saved light/dark theme with full parity.

The look is currently stock Tailwind: no custom tokens, no custom fonts, no motion beyond color transitions. This document records that incumbent honestly. It does not claim a brand voice the code doesn't yet have.

**Key Characteristics:**
- Neutral-first: greys carry structure, blue carries state.
- Flat: hairline borders and tonal fills, no shadows apart from the selected tab.
- Compact: 14px body, 8-16px gaps, small radii.
- Native system-ui type; tabular numerals for the countdown.
- Dark mode is a real theme (`[data-theme="dark"]`), not an inversion afterthought.

## Colors

A neutral grey scale with one saturated blue accent and three semantic status colors used sparingly.

### Primary
- **Switch Blue** (#2563eb): The only accent. Fills an enabled toggle, a selected timer chip, the active settings tab underline and the primary Save/Start buttons. Hover deepens to **Deep Switch Blue** (#1d4ed8).

### Secondary
- **Live Green** (#22c55e): The 6px dot on a feature tab meaning "this feature is running". Text-sized status uses #16a34a.
- **Pause Amber** (#d97706): Paused state and the Resume action text.
- **Fault Red** (#ef4444): Inline shortcut errors only.

### Neutral
- **Paper** (#ffffff) / **Night Paper** (#171717): Window background.
- **Soft Fill** (#f5f5f5) / **Night Fill** (#262626): Tab-strip well and unselected chips. Hover steps to #e5e5e5.
- **Raised Night** (#404040): Selected tab in dark mode.
- **Hairline** (#e5e5e5, strong #d4d4d4) / **Night Hairline** (#404040): Borders on sections, buttons and inputs.
- **Ink** (#171717) / **Night Ink** (#f5f5f5): Primary text.
- **Muted Ink** (#737373): Status lines, labels, hints, inactive tabs.
- **Switch Off** (#d4d4d4) / **Night Switch Off** (#525252): Toggle track at rest.

### Named Rules
**The Blue Means State Rule.** Blue appears only on something that is on, selected or primary. It is never used for decoration, headings or links.

**The Green Dot Rule.** Green is a presence indicator (a 6px dot), never a button or a fill.

## Typography

**Display Font:** none. **Body Font:** system-ui (with sans-serif fallback). **Label/Mono Font:** none; numerals use tabular figures.

**Character:** Whatever the OS speaks. Hierarchy comes from weight (400 / 500 / 600) and a neutral muted tone, not from size contrast.

### Hierarchy
- **Title** (600, 1.25rem, 1.4): Settings window page heading.
- **Readout** (600, 1.5rem, tabular-nums): The countdown, the one big number in the product.
- **Body** (400, 14px, 1.4): Default text, feature names (at 600 in the panel).
- **Label** (500, 0.875rem): Buttons, chips, tab labels.
- **Caption** (400, 0.75rem, muted): Hints and inline errors.

### Named Rules
**The Weight Over Size Rule.** Emphasis is bold weight or muted color. Do not add new sizes to make hierarchy.

## Layout

The tray Panel is a fixed-height flex column with 16px outer padding and 12px between sections. The pinned footer (Settings, Quit) sits at the bottom in a two-column grid. Timer chips are a three-column grid with 8px gaps. Settings is a single centered column, `max-w-xl`, 20px padding, sections stacked on a 16px rhythm.

Spacing is Tailwind's 4px scale, used at 4 / 8 / 12 / 16 / 20. Density is compact. The Panel never scrolls its primary controls, and the UI selects no text (`select-none`) because it behaves like a control surface.

## Elevation & Depth

Flat. Depth comes from hairline borders and tonal fills: a soft fill well holds the feature tabs, and a bordered rounded section holds each feature. The one exception is a small default shadow on the selected feature tab, which lifts it out of the well. Nothing else casts a shadow, and there is no blur or glass.

### Named Rules
**The Flat-By-Default Rule.** Surfaces are flat and bordered. Add a shadow only to show that a segmented-control item is the selected one.

## Shapes

Small, consistent rounding: 12px on sections and the tab well, 8px on buttons, chips and tab items, 4px on settings inputs and buttons, full pills for the toggle track and knob. Borders are always 1px hairlines. No decorative geometry.

## Components

### Buttons
- **Shape:** 8px radius in the Panel, 4px in Settings.
- **Secondary (Panel):** Transparent with a 1px hairline border, 8px 12px padding, hover fills Soft Fill. Used for pause durations, Settings, Quit.
- **Primary (Settings):** Switch Blue fill, white text, 4px 12px padding, hover Deep Switch Blue. Used for Save and Start.
- **Disabled:** 50% opacity.

### Chips
- **Style:** Soft Fill background, label weight, 8px radius, 8px padding, in a 3-column grid.
- **State:** Selected is Switch Blue with white text, but only while the feature is running.

### Cards / Containers
- **Corner Style:** 12px in Panel, 8px in Settings feature cards.
- **Background:** Same as the window; separation is the hairline border.
- **Internal Padding:** 12px (Panel), 16px (Settings).

### Inputs / Fields
- **Style:** Transparent background, 1px hairline border, 4px radius, 4px 8px padding. Shortcut capture fields are read-only and centered.
- **Focus:** Browser default; no custom treatment yet.
- **Error:** Caption-size red text beneath the field.

### Navigation
- **Panel feature tabs:** a segmented control in a Soft Fill well; the selected item is white (Raised Night in dark) with a small shadow and semibold text; inactive items are muted.
- **Settings tabs:** underline tabs on a hairline rule; active is semibold with a 2px Switch Blue bottom border.

### Toggle (signature)
A 44x24 pill track (Switch Off at rest, Switch Blue when on) with a 20px white knob that slides 20px. The native checkbox is visually hidden but remains the control, so it is keyboard and screen-reader accessible.

## Do's and Don'ts

### Do:
- **Do** keep blue for on / selected / primary only.
- **Do** support both themes for every new surface using the `dark:` variant tied to `[data-theme="dark"]`.
- **Do** use hairline borders and tonal fills for separation.
- **Do** keep strings translatable: layouts must tolerate longer Türkçe labels.
- **Do** keep the countdown in tabular numerals.

### Don't:
- **Don't** add shadows, glass or gradients; the system is flat.
- **Don't** introduce a second accent color.
- **Don't** add attention-seeking motion, badges or sounds; the product stays out of the way.
- **Don't** use green as a fill or button.
- **Don't** hardcode user-facing strings; every string needs English and Türkçe.

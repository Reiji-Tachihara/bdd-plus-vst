# BDD+ Agent Guide

## Goal

Implement the core distortion DSP for the "BDD+" VST3 plugin (Cubase 13 on Windows).
We want a "BD-2 style" overdrive vibe: dynamic, touch-sensitive, not overly fuzzy, with a pleasant high-end.

## Target sound notes (BD-2-ish)

- Keep low end tight (avoid boomy distortion)
- Preserve pick attack; clean-ish when Drive low
- At higher Drive: saturated but not square/fuzzy
- Tone at 0.5 should be balanced; 0.0 darker, 1.0 brighter but not ice-picky

## Plugin context

- Framework: nih-plug (Rust)
- Format: VST3
- GUI: already exists or will exist separately (Drive/Tone/Level knobs)

## Parameters

- Drive: 0.0 .. 1.0 (maps to input gain / saturation amount)
- Tone: 0.0 .. 1.0 (maps to post-EQ / tone filter)
- Level: 0.0 .. 1.0 (output gain)

Provide recommended parameter-to-dsp mappings (log/exp curves where appropriate).

## Parameter resolution

- Internal parameter values must remain continuous (f32).
- GUI interaction should use ~24 steps for Drive/Tone/Level.
- Automation must stay smooth (no stepping in DSP).

## DSP architecture (must implement in this order)

1. Input trim / pre-emphasis filter (to avoid flubby low end before clipping)
2. Oversampling (start with 2x; optional 4x later)
3. Nonlinearity (waveshaper)
   - Start with simple soft clip (tanh or polynomial)
   - Then add subtle asymmetry option (BD-2-ish character)
4. Post filter / Tone control
   - Simple one-knob tone that shapes highs without harshness
5. Output level

## Real-time safety constraints (very important)

- No heap allocations in process()
- No locking/mutex in process()
- No logging/println in process()
- No denormals issues (use small noise or flush-to-zero if needed)
- Must be stable for stereo processing

## Coding constraints

- Keep DSP code self-contained and testable (separate module for DSP)
- Prefer small structs with explicit state (filters, oversampler buffers)
- Add Japanese comments that explain _why_ each step exists and _What_ is this code doing here
- Provide a minimal unit test or offline test function where possible

## Deliverables

- Rust code changes implementing the DSP path
- Clear instructions which files were edited and why
- A short "how to tune" section: what to change to get closer to BD-2 character
- If unsure about exact nih-plug API usage, ask for current project file structure
  (or infer from common nih-plug template layout)

## Acceptance criteria

- Build succeeds: `cargo xtask bundle bdd_plus --release`
- In Cubase, audio passes and Drive/Tone/Level audibly work
- No obvious aliasing at moderate drive (2x oversampling baseline)
- No crashes / no extreme CPU spikes

## NyashkChannel 1.0.3

- **Display fix:** the built-in CPU renderer (used when Vulkan is not available) drew pixels on the shared edges of triangles twice, which showed as brighter seams and specks in see-through fills (spectra, curves, bars). It now uses an exact fill rule like a GPU: every pixel is drawn once, without holes.
- Ratio: typing 19.5:1 no longer jumps to INF.
- Clicking a value box and pressing Enter no longer changes a value whose unit matters (for example "1.03 s" was read back as 1.03 ms). This affected the expander and compressor Release.
- No more "-0.0" read-outs. (Output)
- New automatic test: every parameter's text is typed back over its whole range and must give the same value.

## NyashkChannel 1.0.2

- **Filters on the graph:** the HP and LP filters are now part of the EQ curve, with markers and their frequencies (dashed when SPLIT sends them to the dynamics sidechain only).
- **DYN view:** a second graph view shows the expander / gate and compressor curve (output level against input level) with the thresholds and the live level moving on the curve. It opens by itself while you turn a dynamics knob; EQ / DYN buttons switch by hand.
- Value boxes have a gap between them, and a value that would not fit its box is drawn smaller instead of running over the frame.
- Gain reduction read-outs show 0.0 instead of -0.0.

## NyashkChannel 1.0.1

- **Live spectrum:** the output spectrum now runs behind the EQ curve.

## NyashkChannel 1.0.0

First release. VST3 for Linux, Windows and macOS.

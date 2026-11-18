## NyashkChannel 1.0.11

- Деталі дизайну: стрілка ручки починається від краю ковпачка (як справжня гравіровка), бокові стінки металевих ручок ловлять світло, навколо ручки, яку крутиш або на яку наводиш, світиться тонкий обідок, LED-кільця без зайвих рисок на маленьких ручках, мітки пікового рівня на LED-шкалах, більші підписи.

## NyashkChannel 1.0.10

- Новий дизайн: сучасне аналогове залізо. Корпус, ручки, кнопки й екрани відрендерені як справжні фізичні об'єкти: точені алюмінієві ручки з рифленою стінкою, тіні в три шари, анодований метал з подряпинами, пилом і потертими краями, кнопки з підсвіткою, LED-дуги навколо ручок, скло на екранах.
- Ручки: тягни вгору / вниз (Shift — точно), колесо миші, подвійний клік — значення за замовчуванням, правий клік — ввести число. Значення показується при наведенні.

## NyashkChannel 1.0.9

- Оверсемплінг тепер рівний до 20 кГц на 44.1 кГц (новий поліфазний half-band фільтр, дзеркала > 100 дБ нижче). Раніше на 44.1 кГц верх був приглушений: −0.6 дБ на 18 кГц і ≈ −5 дБ на 20 кГц.
- Захист від NaN / inf від хоста: один «битий» семпл більше не ламає плагін назавжди (раніше фільтри залишались у NaN і звук пропадав до перезапуску).

## NyashkChannel 1.0.8

- Дзвони на високих частотах тепер мають ту саму форму, що й аналогові, аж до Найквіста: на 44.1/48 кГц високий дзвін більше не «стискається» (cramping). Наприклад, дзвін на 12 кГц +12 дБ дає на 16 кГц +8.5 дБ, і на 96 кГц теж +8.6 дБ (раніше на 44.1 кГц було +4.4 дБ).

## NyashkChannel 1.0.7

- **Click-free switches:** HP / LP, EQ, ANALOG, the expander / gate and the compressor on-off buttons used to cut over at once (the compressor turned off released up to its full reduction in a single sample, the filters restarted from a stale state). Every switch now fades over 10 ms and the filters keep running underneath; checked for every switch with a sharp-corner detector.
- New behaviour tests: the compressor follows its drawn curve at 2:1 / 4:1 / 10:1, attack ~8 ms (FAST ~0.4 ms), release matches the knob, gate removes its range below the threshold and leaves the signal alone above it, PAD -20 dB, phase, mono, SPLIT keys the dynamics without filtering the audio.

## NyashkChannel 1.0.6

- Level scale shows a plain 0 instead of +0.

## NyashkChannel 1.0.5

- Layout: the labels under the Line / Mic knobs and the EQ Q knobs no longer run into the panel borders; every panel has room at the bottom (window 1000 x 654).
- A window size saved by an older version is opened with the new proportions.

## NyashkChannel 1.0.4

- **Alt-tab fix (Linux):** after alt-tab, minimising or covering the window, parts of the picture (backgrounds, knobs, images) could stay blank until something on them moved. The window is now drawn again in full whenever the system throws its contents away, and pictures are no longer lost while the window is hidden or a frame is skipped.
- Smoother images in the CPU renderer (used when Vulkan is not available): pictures drawn smaller than their real size, such as knobs and photos, are now scaled with mipmaps instead of picking single pixels, so they no longer look grainy or jagged.

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

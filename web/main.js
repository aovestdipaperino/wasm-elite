import init, { Elite } from "./pkg/elite_wasm.js";

// Key codes: must match elite_sim::Key's discriminant order.
const K = {
  RollLeft: 0, RollRight: 1, PitchUp: 2, PitchDown: 3, SpeedUp: 4, SlowDown: 5, Fire: 6, Jump: 7,
  Hyperspace: 8, Dock: 9, ViewFront: 10, ViewRear: 11, ViewLeft: 12, ViewRight: 13, Launch: 14,
  Buy: 15, Sell: 16, Equip: 17, GalacticChart: 18, ShortRangeChart: 19, SystemData: 20, Market: 21,
  Status: 22, Inventory: 23, Up: 24, Down: 25, Left: 26, Right: 27, Select: 28, Back: 29, FindHome: 30,
  Digit0: 31, Digit1: 32, Digit2: 33, Digit3: 34, Digit4: 35, Digit5: 36, Digit6: 37, Digit7: 38,
  Digit8: 39, Digit9: 40,
};

// KeyboardEvent.code -> list of game keys. Digits double as screen selectors and equip choices.
const KEYMAP = {
  Comma: [K.RollLeft], Period: [K.RollRight], ArrowLeft: [K.RollLeft, K.Left], ArrowRight: [K.RollRight, K.Right],
  KeyX: [K.PitchUp], KeyS: [K.PitchDown], ArrowDown: [K.PitchUp, K.Down], ArrowUp: [K.PitchDown, K.Up],
  Space: [K.SpeedUp, K.Select], Slash: [K.SlowDown], KeyA: [K.Fire], KeyJ: [K.Jump], KeyH: [K.Hyperspace],
  KeyC: [K.Dock], F1: [K.ViewFront], F2: [K.ViewRear], F3: [K.ViewLeft], F4: [K.ViewRight],
  Digit1: [K.Launch, K.Digit1], Digit2: [K.Market, K.Buy, K.Digit2], Digit3: [K.Market, K.Sell, K.Digit3],
  Digit4: [K.Equip, K.Digit4], Digit5: [K.GalacticChart, K.Digit5], Digit6: [K.ShortRangeChart, K.Digit6],
  Digit7: [K.SystemData, K.Digit7], Digit8: [K.Market, K.Digit8], Digit9: [K.Status, K.Digit9],
  Digit0: [K.Inventory, K.Digit0], KeyB: [K.Buy], KeyO: [K.FindHome], Enter: [K.Select], Escape: [K.Back],
};

const PALETTE = ["#ffffff", "#ff3030", "#ffd75e", "#4fd8ff", "#5eff7a", "#707070"];
const SAVE_KEY = "wasm-elite.commander";

const canvas = document.getElementById("screen");
const ctx = canvas.getContext("2d", { alpha: false });
const statusEl = document.getElementById("status");

await init();
const elite = new Elite();
try {
  const saved = localStorage.getItem(SAVE_KEY);
  if (saved && elite.load(saved)) statusEl.textContent = "Commander loaded from this browser.";
} catch (_) { /* storage unavailable */ }

// Rasterise the font once, one sprite per colour.
const fontBits = elite.font();
const glyphSheets = PALETTE.map((colour) => {
  const sheet = document.createElement("canvas");
  sheet.width = 64 * 8; sheet.height = 8;
  const g = sheet.getContext("2d");
  g.fillStyle = colour;
  for (let i = 0; i < 64; i++) for (let row = 0; row < 8; row++) {
    const bits = fontBits[i * 8 + row];
    for (let col = 0; col < 8; col++) if (bits & (0x80 >> col)) g.fillRect(i * 8 + col, row, 1, 1);
  }
  return sheet;
});

// Keyboard: track "S" in the market as Sell only when the market is showing; the sim ignores
// keys that do not apply, so sending both meanings is harmless.
const held = new Set();
function onKey(e, down) {
  const codes = KEYMAP[e.code];
  if (!codes) return;
  e.preventDefault();
  if (down && held.has(e.code)) return; // ignore auto-repeat
  if (down) held.add(e.code); else held.delete(e.code);
  for (const c of codes) elite.key(c, down);
  // S is both "dive" and "sell"; sell only makes sense as an edge on the market screen.
  if (e.code === "KeyS") elite.key(K.Sell, down);
}
window.addEventListener("keydown", (e) => onKey(e, true));
window.addEventListener("keyup", (e) => onKey(e, false));
window.addEventListener("blur", () => { for (const code of held) for (const c of KEYMAP[code]) elite.key(c, false); held.clear(); });

document.getElementById("fullscreen").onclick = () => canvas.requestFullscreen?.();
document.getElementById("help").onclick = () => { const k = document.getElementById("keys"); k.hidden = !k.hidden; };
document.getElementById("reset").onclick = () => {
  if (!confirm("Start a new commander? The saved one in this browser will be replaced when you next dock.")) return;
  elite.reset();
  try { localStorage.removeItem(SAVE_KEY); } catch (_) {}
};

function draw() {
  const flash = elite.flash();
  ctx.fillStyle = flash ? "#602020" : "#000";
  ctx.fillRect(0, 0, 256, 248);
  const fills = elite.fills();
  for (let i = 0; i < fills.length; i += 5) {
    ctx.fillStyle = PALETTE[fills[i + 4]] || PALETTE[0];
    ctx.fillRect(fills[i], fills[i + 1], fills[i + 2], fills[i + 3]);
  }
  const dots = elite.dots();
  for (let i = 0; i < dots.length; i += 3) {
    ctx.fillStyle = PALETTE[dots[i + 2]] || PALETTE[0];
    ctx.fillRect(dots[i], dots[i + 1], 1, 1);
  }
  const lines = elite.lines();
  // Group by colour to cut down on path state changes.
  for (let colour = 0; colour < PALETTE.length; colour++) {
    ctx.strokeStyle = PALETTE[colour];
    ctx.lineWidth = 1;
    ctx.beginPath();
    let any = false;
    for (let i = 0; i < lines.length; i += 5) {
      if (lines[i + 4] !== colour) continue;
      any = true;
      ctx.moveTo(lines[i] + 0.5, lines[i + 1] + 0.5);
      ctx.lineTo(lines[i + 2] + 0.5, lines[i + 3] + 0.5);
    }
    if (any) ctx.stroke();
  }
  const glyphs = elite.glyphs();
  for (let i = 0; i < glyphs.length; i += 4) {
    const sheet = glyphSheets[glyphs[i + 3]] || glyphSheets[0];
    ctx.drawImage(sheet, glyphs[i + 2] * 8, 0, 8, 8, glyphs[i], glyphs[i + 1], 8, 8);
  }
}

const stepMs = 1000 / elite.step_hz();
let last = performance.now();
let acc = 0;
function loop(now) {
  acc += Math.min(now - last, 250);
  last = now;
  let stepped = false;
  while (acc >= stepMs) {
    if (elite.step()) {
      try { localStorage.setItem(SAVE_KEY, elite.save()); } catch (_) {}
    }
    acc -= stepMs;
    stepped = true;
  }
  if (stepped) {
    draw();
    statusEl.textContent = elite.status();
  }
  requestAnimationFrame(loop);
}
canvas.focus();
requestAnimationFrame(loop);

"use strict";

const copy = {
  zh: {
    flowsheet: "流程图", canvas: "画布", layer: "图层", topology: "连接方向",
    molar: "摩尔流量", temperature: "流股温度", pressure: "流股绝对压力",
    diagramLabel: "Feed–Valve–Flash 流程图", streamResults: "流股结果", displayUnit: "项目显示单位",
    current: "当前快照", equal: "等压快照", zero: "当前快照 · 零流量支路",
    rangeCheck: "模拟数据 · 零流量压力为极值", partial: "模拟数据 · 压力缺值",
    stale: "结果已过期", missing: "尚无结果", failed: "运行失败",
    run: "运行", priorRun: "上次运行", simulated: "画板模拟", topologyNote: "箭头表示连接方向。",
    scaleNote: "范围取自当前快照。", equalNote: "有效值相同，使用同一颜色。",
    includeNote: "带色虚线表示零流量，记录压力参与标尺。",
    excludeNote: "中性虚线表示零流量，记录压力仅在流股结果显示。",
    missingPressure: "压力缺失", missingNote: "运行流程后可显示当前结果。",
    staleNote: "输入已变更至 r1，旧读数和色带已撤下。",
    failedNote: "本次运行未生成可着色的结果。",
    zeroFlow: "零流量", unavailable: "—", issueAction: "查看 Valve 诊断",
    issueTitle: "Valve · 出口压力", issueBody: "出口压力设为 710 kPa，高于入口的 700 kPa。",
    returnToPort: "返回画布标记", review: "画板评审", scenario: "结果场景", language: "语言",
    zeroPolicy: "零流量压力", include: "着色并计入标尺（建议）", exclude: "仅显示读数",
  },
  en: {
    flowsheet: "Flowsheet", canvas: "Canvas", layer: "Layer", topology: "Connection direction",
    molar: "Molar flow", temperature: "Stream temperature", pressure: "Stream absolute pressure",
    diagramLabel: "Feed–Valve–Flash flowsheet", streamResults: "Stream results", displayUnit: "Project display unit",
    current: "Current snapshot", equal: "Equal-pressure snapshot", zero: "Current snapshot · zero-flow branch",
    rangeCheck: "Simulated · zero-flow extreme", partial: "Simulated · missing pressure",
    stale: "Stale result", missing: "No result", failed: "Run failed",
    run: "Run", priorRun: "Previous run", simulated: "Board simulation", topologyNote: "Arrows show connection direction.",
    scaleNote: "Range from the current snapshot.", equalNote: "Equal values share one color.",
    includeNote: "Colored dashes mark zero flow. Its recorded pressure counts in the scale.",
    excludeNote: "Neutral dashes mark zero flow. Its recorded pressure appears in stream results.",
    missingPressure: "No pressure", missingNote: "Run the flowsheet to see current results.",
    staleNote: "Inputs changed to r1. Previous values and the color scale are hidden.",
    failedNote: "This run produced no result for coloring.",
    zeroFlow: "Zero flow", unavailable: "—", issueAction: "Show Valve diagnostic",
    issueTitle: "Valve · outlet pressure", issueBody: "The outlet setting of 710 kPa exceeds the inlet pressure of 700 kPa.",
    returnToPort: "Back to canvas marker", review: "Board review", scenario: "Result scenario", language: "Language",
    zeroPolicy: "Zero-flow pressure", include: "Color + scale (recommended)", exclude: "Show reading only",
  },
};

const streams = [
  { id: "feed", name: "Feed" },
  { id: "throttled", name: "Valve Outlet" },
  { id: "liquid", name: "Liquid Outlet" },
  { id: "vapor", name: "Vapor Outlet" },
];
// Stream order follows the public example. Values come from headless result reads in SI.
const snapshots = {
  current: { revision: 0, pressure: [700000, 650000, 650000, 650000], molar: [5, 5, 4.258037097752094, 0.7419629022479057], temperature: [300, 300, 300, 300] },
  zero: { revision: 1, pressure: [700000, 650000, 650000, 650000], molar: [5, 5, 0, 5], temperature: [300, 300, 345, 345] },
  equal: { revision: 2, pressure: [700000, 700000, 700000, 700000], molar: [5, 5, 5, 0], temperature: [300, 300, 300, 300] },
};
const scenarios = ["current", "zero", "equal", "rangeCheck", "partial", "stale", "missing", "failed"];
const layers = ["topology", "molar", "temperature", "pressure"];
const pressureUnits = { Pa: 1, kPa: 1e3, MPa: 1e6, bar: 1e5 };
const params = new URLSearchParams(location.search);
const option = (param, values, fallback) => values.includes(params.get(param)) ? params.get(param) : fallback;
const state = {
  scenario: option("state", scenarios, "current"),
  language: option("lang", ["zh", "en"], "zh"),
  layer: option("layer", layers, "pressure"),
  unit: option("unit", Object.keys(pressureUnits), "kPa"),
  policy: option("policy", ["include", "exclude"], "include"),
  selected: option("stream", streams.map(({ id }) => id), "throttled"),
  issueOpen: params.get("issue") === "1",
};

const $ = (id) => document.getElementById(id);
const t = (key) => copy[state.language][key];
const neutral = "#67798a";
const low = [109, 129, 155];
const high = [24, 73, 136];
const canvas = $("lines");
const ctx = canvas.getContext("2d");
const number = new Intl.NumberFormat("en", { maximumFractionDigits: 6 });
const controls = new Map();

for (const { id, name } of streams) {
  const label = document.createElement("button");
  label.type = "button";
  label.className = "line-label";
  label.dataset.stream = id;
  $("stream-labels").append(label);

  const row = document.createElement("li");
  const button = document.createElement("button");
  button.type = "button";
  button.className = "result-row";
  button.dataset.stream = id;
  const rowName = document.createElement("span");
  rowName.className = "result-name";
  const swatch = document.createElement("span");
  swatch.className = "swatch";
  swatch.setAttribute("aria-hidden", "true");
  rowName.append(swatch, name);
  const value = document.createElement("span");
  value.className = "result-value";
  button.append(rowName, value);
  row.append(button);
  $("stream-results").append(row);
  controls.set(id, { label, button, value, swatch });
  for (const control of [label, button]) {
    control.addEventListener("click", () => {
      state.selected = id;
      render();
    });
  }
}

for (const key of ["layer", "scenario", "language", "unit", "policy"]) {
  $(key).addEventListener("change", (event) => {
    if (key === "scenario") state.issueOpen = false;
    state[key] = event.target.value;
    render();
  });
}
$("issue-pin").addEventListener("click", () => {
  state.issueOpen = true;
  render();
  $("issue-title").focus();
});
$("back-to-issue").addEventListener("click", () => {
  state.issueOpen = false;
  render();
  $("issue-pin").focus();
});

function snapshotFor(scenario) {
  // These two board states isolate range and missing-value behavior.
  if (scenario === "rangeCheck") return { ...snapshots.zero, pressure: [700000, 650000, 600000, 650000] };
  if (scenario === "partial") return { ...snapshots.current, pressure: [700000, 650000, 650000, null] };
  return snapshots[scenario];
}

function reading(snapshot, index) {
  if (!snapshot || state.layer === "topology") return { value: null, mapped: null, zeroFlow: false, dashed: false };
  const zeroFlow = snapshot.molar[index] === 0;
  const value = snapshot[state.layer][index];
  const pressure = state.layer === "pressure";
  const excluded = zeroFlow && (state.layer === "temperature" || (pressure && state.policy === "exclude"));
  return {
    value: state.layer === "temperature" && excluded ? null : value,
    mapped: excluded ? null : value,
    zeroFlow,
    dashed: (pressure || state.layer === "temperature") && (zeroFlow || value === null),
  };
}

function display(value) {
  if (state.layer === "pressure") return `${number.format(value / pressureUnits[state.unit])} ${state.unit}`;
  if (state.layer === "temperature") return `${number.format(value)} K`;
  return `${number.format(value)} mol/s`;
}

function scaleColor(value, minimum, maximum) {
  if (value === null) return neutral;
  const ratio = minimum === maximum ? 0.5 : (value - minimum) / (maximum - minimum);
  const color = low.map((v, index) => Math.round(v + (high[index] - v) * ratio));
  return `rgb(${color.join(", ")})`;
}

function render() {
  document.documentElement.lang = state.language === "zh" ? "zh-CN" : "en";
  document.title = `RadishFlow Studio · ${t("pressure")}`;
  for (const item of document.querySelectorAll("[data-text]")) item.textContent = t(item.dataset.text);
  for (const item of document.querySelectorAll("[data-label]")) item.setAttribute("aria-label", t(item.dataset.label));
  $("scenario").replaceChildren(...scenarios.map((scenario) => new Option(t(scenario), scenario)));
  for (const key of ["layer", "scenario", "language", "unit", "policy"]) $(key).value = state[key];

  const snapshot = snapshotFor(state.scenario);
  const pressure = state.layer === "pressure";
  const topology = state.layer === "topology";
  const failed = state.scenario === "failed";
  const readings = streams.map((_, index) => reading(snapshot, index));
  const values = readings.map(({ mapped }) => mapped).filter((value) => value !== null);
  const showScale = values.length > 0;
  const minimum = showScale ? (state.layer === "molar" ? 0 : Math.min(...values)) : null;
  const maximum = showScale ? Math.max(...values) : null;
  const equal = showScale && minimum === maximum;

  $("unit").closest(".unit-control").hidden = !pressure;
  $("status").textContent = t(state.scenario);
  $("status").dataset.state = snapshot ? "current" : state.scenario;
  $("legend-title").textContent = t(state.layer);
  const simulated = ["rangeCheck", "partial"].includes(state.scenario);
  const origin = snapshot ? `${t("run")} #1 · r${snapshot.revision}` :
    state.scenario === "stale" ? `${t("priorRun")} #1 · r0` : failed ? `${t("run")} #2 · r1` : "";
  $("origin").textContent = simulated ? t("simulated") : origin;
  $("scale").hidden = !showScale;
  $("scale").querySelector(".ramp").dataset.equal = String(equal);
  $("scale").querySelector(".bounds").dataset.equal = String(equal);
  $("min").textContent = showScale ? display(minimum) : "";
  $("max").textContent = showScale && !equal ? display(maximum) : "";
  $("max").hidden = equal;
  const note = { stale: "staleNote", missing: "missingNote", failed: "failedNote" }[state.scenario];
  $("layer-note").textContent = t(topology ? "topologyNote" : (note ?? (equal ? "equalNote" : "scaleNote")));
  $("layer-note-detail").textContent = pressure && readings.some(({ zeroFlow }) => zeroFlow)
    ? t(state.policy === "include" ? "includeNote" : "excludeNote") : "";
  $("issue-pin").hidden = !failed;
  $("issue-detail").hidden = !(failed && state.issueOpen);

  for (const [index, { id, name }] of streams.entries()) {
    const { label, button, value: rowValue, swatch } = controls.get(id);
    const { value, mapped, zeroFlow, dashed } = readings[index];
    const text = value !== null ? display(value) : snapshot && pressure ? t("missingPressure") : t("unavailable");
    const labelText = mapped !== null ? text : t("unavailable");
    const stroke = scaleColor(mapped, minimum, maximum);
    for (const control of [label, button]) control.setAttribute("aria-pressed", String(id === state.selected));
    label.hidden = !snapshot || topology;
    label.textContent = labelText;
    label.setAttribute("aria-label", [name, text, zeroFlow ? t("zeroFlow") : ""].filter(Boolean).join(" · "));
    label.dataset.stroke = stroke;
    label.dataset.dashed = String(dashed);
    swatch.style.setProperty("--stroke", stroke);
    swatch.style.backgroundColor = dashed ? "" : stroke;
    swatch.dataset.dashed = String(dashed);
    rowValue.textContent = topology ? "" : text;
    if (zeroFlow && !topology) {
      const detail = document.createElement("small");
      detail.textContent = t("zeroFlow");
      rowValue.append(detail);
    }
  }
  drawDiagram();
}

function geometry(diagram) {
  const box = (id) => {
    const rect = $(id).getBoundingClientRect();
    const x = rect.x - diagram.x;
    const y = rect.y - diagram.y;
    return { x, y, right: x + rect.width, bottom: y + rect.height, cx: x + rect.width / 2, cy: y + rect.height / 2 };
  };
  const feed = box("feed");
  const valve = box("valve");
  const flash = box("flash");
  if (matchMedia("(max-width: 575px)").matches) {
    const outgoing = (sign) => {
      const x = flash.cx + sign * 13;
      const endX = diagram.width * (sign < 0 ? .24 : .76);
      const bendY = diagram.height * .83;
      return { points: [[x, flash.bottom], [x, bendY], [endX, bendY], [endX, diagram.height * .96]], label: [endX, bendY - 18] };
    };
    return {
      feed: { points: [[feed.cx, feed.bottom], [valve.cx, valve.y]], label: [feed.cx + 63, (feed.bottom + valve.y) / 2] },
      throttled: { points: [[valve.cx, valve.bottom], [flash.cx, flash.y]], label: [valve.cx + 63, (valve.bottom + flash.y) / 2] },
      liquid: outgoing(-1), vapor: outgoing(1), issue: [valve.right + 22, valve.cy],
    };
  }
  const outgoing = (sign) => {
    const y = flash.cy + sign * 9;
    const endY = flash.cy + sign * Math.min(60, diagram.height * .22);
    const bendX = flash.right + (diagram.width - flash.right) * .34;
    return {
      points: [[flash.right, y], [bendX, y], [bendX, endY], [diagram.width - 15, endY]],
      label: [diagram.width * .88, endY + sign * 22],
    };
  };
  return {
    feed: { points: [[feed.right, feed.cy], [valve.x, valve.cy]], label: [(feed.right + valve.x) / 2, feed.cy - 25] },
    throttled: { points: [[valve.right, valve.cy], [flash.x, flash.cy]], label: [(valve.right + flash.x) / 2, valve.cy - 25] },
    liquid: outgoing(1), vapor: outgoing(-1), issue: [valve.cx, valve.bottom + 19],
  };
}

function drawDiagram() {
  const diagram = $("diagram").getBoundingClientRect();
  const ratio = devicePixelRatio || 1;
  canvas.width = Math.round(diagram.width * ratio);
  canvas.height = Math.round(diagram.height * ratio);
  ctx.setTransform(ratio, 0, 0, ratio, 0, 0);
  ctx.lineJoin = "round";
  ctx.lineCap = "round";
  const graph = geometry(diagram);
  for (const { id } of streams) {
    const { points, label } = graph[id];
    const labelButton = controls.get(id).label;
    const { dashed, stroke } = labelButton.dataset;
    if (state.selected === id) strokePath(points, "#c8dbee", 9);
    if (dashed === "true") ctx.setLineDash([5, 5]);
    strokePath(points, stroke, 3);
    ctx.setLineDash([]);
    const [endX, endY] = points.at(-1);
    const [prevX, prevY] = points.at(-2);
    const angle = Math.atan2(endY - prevY, endX - prevX);
    const back = (delta) => [endX - 8 * Math.cos(angle + delta), endY - 8 * Math.sin(angle + delta)];
    strokePath([back(-.55), [endX, endY], back(.55)], stroke, 2.5);
    place(labelButton, label);
  }
  place($("issue-pin"), graph.issue);
}

function strokePath(points, stroke, width) {
  ctx.beginPath();
  points.forEach(([x, y], index) => index ? ctx.lineTo(x, y) : ctx.moveTo(x, y));
  ctx.lineWidth = width;
  ctx.strokeStyle = stroke;
  ctx.stroke();
}

function place(element, [x, y]) {
  element.style.left = `${x}px`;
  element.style.top = `${y}px`;
}

document.documentElement.dataset.size = params.get("size") === "20" ? "20" : "14";
$("review").hidden = params.get("review") === "0";
for (const unit of Object.keys(pressureUnits)) $("unit").add(new Option(unit, unit));
render();
new ResizeObserver(drawDiagram).observe($("diagram"));
document.fonts.ready.then(drawDiagram);

"use strict";

const copy = {
  zh: {
    flowsheet: "流程图", canvas: "画布", layer: "图层", topology: "连接方向",
    molarFlow: "摩尔流量", temperature: "流股温度", diagramLabel: "Feed–Heater–Flash 流程图",
    streamResults: "流股结果", displayUnit: "项目显示单位", current: "当前快照",
    heaterOnly: "仅调整 Heater 温度", isothermal: "等温快照", partial: "当前快照 · 温度缺值",
    stale: "结果已过期", missing: "尚无结果", failed: "运行失败",
    run: "运行", priorRun: "上次运行", topologyNote: "箭头表示连接方向。",
    scaleNote: "范围取自当前快照。", isothermalNote: "有效温度相同，使用同一颜色。",
    zeroFlowNote: "零流量流股保留连接方向，以中性虚线显示。",
    partialNote: "Vapor Outlet 缺温度，保留连接方向。",
    staleNote: "输入已变更至 r1，旧读数及色带已撤下。",
    missingNote: "运行流程后可显示当前结果。", failedNote: "本次运行未生成可着色的结果。",
    zeroFlow: "零流量", noTemperature: "温度缺失", unavailable: "—",
    issueAction: "查看 Heater 入口诊断", issueTitle: "Heater · inlet", issueBody: "入口端口未连接。",
    returnToPort: "返回画布标记", review: "画板评审", scenario: "结果场景", language: "语言",
  },
  en: {
    flowsheet: "Flowsheet", canvas: "Canvas", layer: "Layer", topology: "Connection direction",
    molarFlow: "Molar flow", temperature: "Stream temperature", diagramLabel: "Feed–Heater–Flash flowsheet",
    streamResults: "Stream results", displayUnit: "Project display unit", current: "Current snapshot",
    heaterOnly: "Heater temperature changed", isothermal: "Isothermal snapshot", partial: "Current snapshot · missing temperature",
    stale: "Stale result", missing: "No result", failed: "Run failed",
    run: "Run", priorRun: "Previous run", topologyNote: "Arrows show connection direction.",
    scaleNote: "Range from the current snapshot.", isothermalNote: "Equal temperatures share one color.",
    zeroFlowNote: "Zero-flow streams keep their arrows and use neutral dashes.",
    partialNote: "Vapor Outlet has no temperature. Its connection remains visible.",
    staleNote: "Inputs changed to r1. Previous values and the color scale are hidden.",
    missingNote: "Run the flowsheet to see current results.", failedNote: "This run produced no result for coloring.",
    zeroFlow: "Zero flow", noTemperature: "No temperature", unavailable: "—",
    issueAction: "Show Heater inlet diagnostic", issueTitle: "Heater · inlet", issueBody: "The inlet port is unbound.",
    returnToPort: "Back to canvas marker", review: "Board review", scenario: "Result scenario", language: "Language",
  },
};

const streams = [
  { id: "feed", name: "Feed", flow: 5 },
  { id: "heated", name: "Heated Outlet", flow: 5 },
  { id: "liquid", name: "Liquid Outlet", flow: 0 },
  { id: "vapor", name: "Vapor Outlet", flow: 5 },
];
// In stream order, from independent headless solves of the public example.
const snapshots = {
  current: { revision: 0, temperatures: [300, 345, 345, 345] },
  heaterOnly: { revision: 1, temperatures: [300, 300, 345, 345] },
  isothermal: { revision: 2, temperatures: [300, 300, 300, 300] },
};
const states = ["current", "heaterOnly", "isothermal", "partial", "stale", "missing", "failed"];
const params = new URLSearchParams(location.search);
const state = {
  scenario: states.includes(params.get("state")) ? params.get("state") : "current",
  language: params.get("lang") === "en" ? "en" : "zh",
  layer: ["molar", "topology"].includes(params.get("layer")) ? params.get("layer") : "temperature",
  temperatureUnit: params.get("unit") === "°C" ? "°C" : "K",
  molarUnit: "mol/s",
  selected: streams.some(({ id }) => id === params.get("stream")) ? params.get("stream") : "heated",
  issueOpen: params.get("issue") === "1",
};
const $ = (id) => document.getElementById(id);
const t = (key) => copy[state.language][key];
const neutral = "#67798a";
const low = [109, 129, 155];
const high = [24, 73, 136];
const canvas = $("lines");
const ctx = canvas.getContext("2d");
const formatNumber = new Intl.NumberFormat("en", { maximumFractionDigits: 3 });
const labels = new Map();
const rows = new Map();

for (const stream of streams) {
  const label = document.createElement("button");
  label.type = "button";
  label.className = "line-label";
  label.dataset.stream = stream.id;
  $("stream-labels").append(label);
  labels.set(stream.id, label);

  const row = document.createElement("li");
  const button = document.createElement("button");
  button.type = "button";
  button.className = "result-row";
  button.dataset.stream = stream.id;
  const name = document.createElement("span");
  name.className = "result-name";
  const swatch = document.createElement("span");
  swatch.className = "swatch";
  swatch.setAttribute("aria-hidden", "true");
  name.append(swatch, stream.name);
  const value = document.createElement("span");
  value.className = "result-value";
  button.append(name, value);
  row.append(button);
  $("stream-results").append(row);
  rows.set(stream.id, { button, value, swatch });
  for (const control of [label, button]) {
    control.addEventListener("click", () => {
      state.selected = stream.id;
      render();
    });
  }
}

for (const key of ["layer", "scenario", "language"]) {
  $(key).addEventListener("change", (event) => {
    if (key === "scenario") state.issueOpen = false;
    state[key] = event.target.value;
    render();
  });
}
$("unit").addEventListener("change", (event) => {
  const unitKey = state.layer === "molar" ? "molarUnit" : "temperatureUnit";
  state[unitKey] = event.target.value;
  render();
});
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

function reading(snapshot, stream, index) {
  if (!snapshot) return { value: null, kind: "unavailable" };
  if (state.layer === "molar") return { value: stream.flow, kind: "value" };
  if (stream.flow === 0) return { value: null, kind: "zero-flow" };
  if (state.scenario === "partial" && stream.id === "vapor") {
    return { value: null, kind: "missing-value" };
  }
  return { value: snapshot.temperatures[index], kind: "value" };
}

function displayAmount(amount) {
  const unit = state.layer === "molar" ? state.molarUnit : state.temperatureUnit;
  let converted = amount;
  if (unit === "kmol/h") converted *= 3.6;
  if (unit === "°C") converted -= 273.15;
  return `${formatNumber.format(converted)} ${unit}`;
}

function scaleColor(amount, minimum, maximum) {
  if (state.layer === "topology" || amount === null) return neutral;
  const ratio = minimum === maximum ? 0.5 : (amount - minimum) / (maximum - minimum);
  return `rgb(${low.map((value, index) => Math.round(value + (high[index] - value) * ratio)).join(", ")})`;
}

function render() {
  document.documentElement.lang = state.language === "zh" ? "zh-CN" : "en";
  document.title = `RadishFlow Studio · ${t("temperature")}`;
  for (const item of document.querySelectorAll("[data-text]")) item.textContent = t(item.dataset.text);
  for (const item of document.querySelectorAll("[data-label]")) item.setAttribute("aria-label", t(item.dataset.label));
  $("scenario").replaceChildren(...states.map((value) => new Option(t(value), value)));
  const molar = state.layer === "molar";
  $("unit").replaceChildren(...(molar ? ["mol/s", "kmol/h"] : ["K", "°C"]).map((unit) => new Option(unit, unit)));
  $("unit").value = molar ? state.molarUnit : state.temperatureUnit;
  $("unit").closest(".unit-control").hidden = state.layer === "topology";
  for (const key of ["layer", "scenario", "language"]) $(key).value = state[key];

  const snapshot = state.scenario === "partial" ? snapshots.current : snapshots[state.scenario];
  const failed = state.scenario === "failed";
  const readings = streams.map((stream, index) => reading(snapshot, stream, index));
  const values = readings.flatMap(({ value }) => value === null ? [] : [value]);
  const minimum = molar ? 0 : Math.min(...values);
  const maximum = Math.max(...values);
  const showScale = values.length > 0 && state.layer !== "topology";
  const equal = showScale && minimum === maximum;
  $("status").textContent = t(snapshot && state.scenario === "heaterOnly" ? "current" : state.scenario);
  $("status").dataset.state = snapshot ? "current" : state.scenario;
  $("legend-title").textContent = t({ molar: "molarFlow", temperature: "temperature", topology: "topology" }[state.layer]);
  $("origin").textContent = snapshot ? `${t("run")} #1 · r${snapshot.revision}` :
    state.scenario === "stale" ? `${t("priorRun")} #1 · r0` : failed ? `${t("run")} #2 · r1` : "";
  $("scale").hidden = !showScale;
  $("scale").dataset.equal = String(equal);
  $("scale").querySelector(".ramp").dataset.equal = String(equal);
  $("scale").querySelector(".bounds").dataset.equal = String(equal);
  $("min").textContent = showScale ? displayAmount(minimum) : "";
  $("max").textContent = showScale && !equal ? displayAmount(maximum) : "";
  $("max").hidden = equal;
  const stateNote = { stale: "staleNote", missing: "missingNote", failed: "failedNote" }[state.scenario];
  $("layer-note").textContent = t(state.layer === "topology" ? "topologyNote" :
    (stateNote ?? (equal ? "isothermalNote" : "scaleNote")));
  $("layer-note-detail").textContent = snapshot && state.layer === "temperature"
    ? [t("zeroFlowNote"), state.scenario === "partial" ? t("partialNote") : ""].filter(Boolean).join(" ")
    : "";
  $("issue-pin").hidden = !failed;
  $("issue-detail").hidden = !(failed && state.issueOpen);
  for (const [index, { id, name }] of streams.entries()) {
    const { value, kind } = readings[index];
    const row = rows.get(id);
    const label = labels.get(id);
    const dataLabel = kind === "zero-flow" ? t("zeroFlow") : kind === "missing-value" ? t("noTemperature") : t("unavailable");
    const hasValue = value !== null && state.layer !== "topology";
    const text = hasValue ? displayAmount(value) : dataLabel;
    const stroke = scaleColor(value, minimum, maximum);
    for (const control of [label, row.button]) control.setAttribute("aria-pressed", String(id === state.selected));
    label.hidden = !showScale;
    label.textContent = hasValue ? text : t("unavailable");
    label.setAttribute("aria-label", `${name} · ${text}`);
    label.dataset.kind = kind;
    label.dataset.stroke = stroke;
    row.swatch.style.backgroundColor = stroke;
    row.value.textContent = state.layer !== "topology" ? text : "";
    row.value.dataset.kind = kind;
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
  const heater = box("heater");
  const flash = box("flash");
  const failed = state.scenario === "failed";
  if (matchMedia("(max-width: 575px)").matches) {
    const port = [heater.cx, heater.y];
    const outgoing = (sign) => {
      const x = flash.cx + sign * 13;
      const endX = diagram.width * (sign < 0 ? .24 : .76);
      const bendY = diagram.height * .83;
      return { points: [[x, flash.bottom], [x, bendY], [endX, bendY], [endX, diagram.height * .96]], label: [endX, diagram.height * .9] };
    };
    return {
      feed: { points: [[feed.cx, feed.bottom], [port[0], port[1] - (failed ? 17 : 0)]], label: [feed.cx + 63, (feed.bottom + heater.y) / 2] },
      heated: { points: [[heater.cx, heater.bottom], [flash.cx, flash.y]], label: [heater.cx + 63, (heater.bottom + flash.y) / 2] },
      liquid: outgoing(-1), vapor: outgoing(1), issue: [heater.right + 24, port[1] - 10], port,
    };
  }
  const port = [heater.x, heater.cy];
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
    feed: { points: [[feed.right, feed.cy], [port[0] - (failed ? 17 : 0), port[1]]], label: [(feed.right + heater.x) / 2, feed.cy - 25] },
    heated: { points: [[heater.right, heater.cy], [flash.x, flash.cy]], label: [(heater.right + flash.x) / 2, heater.cy - 25] },
    liquid: outgoing(1), vapor: outgoing(-1), issue: [port[0] - 23, port[1] + 28], port,
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
    const labelButton = labels.get(id);
    const { kind, stroke } = labelButton.dataset;
    if (state.selected === id) strokePath(points, "#c8dbee", 9);
    if (state.layer === "temperature" && ["zero-flow", "missing-value"].includes(kind)) {
      ctx.setLineDash([5, 5]);
    }
    strokePath(points, stroke, 3);
    ctx.setLineDash([]);
    const [endX, endY] = points.at(-1);
    const [prevX, prevY] = points.at(-2);
    const angle = Math.atan2(endY - prevY, endX - prevX);
    const back = (delta) => [endX - 8 * Math.cos(angle + delta), endY - 8 * Math.sin(angle + delta)];
    strokePath([back(-.55), [endX, endY], back(.55)], stroke, 2.5);
    place(labelButton, label);
  }
  if (state.scenario === "failed") {
    const [x, y] = graph.port;
    ctx.beginPath();
    ctx.arc(x, y, 4, 0, 2 * Math.PI);
    ctx.fillStyle = "#fff";
    ctx.fill();
    ctx.strokeStyle = "#923744";
    ctx.lineWidth = 2;
    ctx.stroke();
    place($("issue-pin"), graph.issue);
  }
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
render();
new ResizeObserver(drawDiagram).observe($("diagram"));
document.fonts.ready.then(drawDiagram);

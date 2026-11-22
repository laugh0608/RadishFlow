"use strict";

const copy = {
  zh: {
    flowsheet: "流程图", canvas: "画布", layer: "图层", topology: "连接方向", molarFlow: "摩尔流量",
    diagramLabel: "Feed–Heater–Flash 流程图", streamResults: "流股结果", displayUnit: "项目显示单位",
    current: "当前快照", partial: "当前快照 · 部分流股缺值", stale: "结果已过期", missing: "尚无结果",
    failed: "运行失败", run: "运行", priorRun: "上次运行", topologyNote: "箭头表示连接方向。",
    scaleNote: "范围取自当前快照。", partialNote: "Vapor Outlet 无流量数据，保留连接方向。",
    staleNote: "输入已变更至 r1，已隐藏旧流量及色带。", missingNote: "运行流程后才可显示流量色带。",
    failedNote: "本次运行未生成可着色的结果。", noValue: "无流量数据", unavailable: "—",
    issueAction: "查看 Heater 入口诊断", issueTitle: "Heater · inlet", issueBody: "入口端口未连接。",
    returnToPort: "返回画布标记", review: "画板评审", scenario: "结果场景", language: "语言",
  },
  en: {
    flowsheet: "Flowsheet", canvas: "Canvas", layer: "Layer", topology: "Connection direction", molarFlow: "Molar flow",
    diagramLabel: "Feed–Heater–Flash flowsheet", streamResults: "Stream results", displayUnit: "Project display unit",
    current: "Current snapshot", partial: "Current snapshot · partial data", stale: "Stale result", missing: "No result",
    failed: "Run failed", run: "Run", priorRun: "Previous run", topologyNote: "Arrows show connection direction.",
    scaleNote: "Range from the current snapshot.", partialNote: "Vapor Outlet has no flow value. Its connection is still shown.",
    staleNote: "Inputs changed to r1. Old values and the scale are hidden.", missingNote: "Run the flowsheet to see the flow scale.",
    failedNote: "This run produced no result for coloring.", noValue: "No flow value", unavailable: "—",
    issueAction: "Show Heater inlet diagnostic", issueTitle: "Heater · inlet", issueBody: "The inlet port is unbound.",
    returnToPort: "Back to canvas marker", review: "Board review", scenario: "Result scenario", language: "Language",
  },
};

// Values read from the public example's r0, sequence 1 solve, in mol/s.
const streams = [
  { id: "feed", name: "Feed", amount: 5 },
  { id: "heated", name: "Heated Outlet", amount: 5 },
  { id: "liquid", name: "Liquid Outlet", amount: 0 },
  { id: "vapor", name: "Vapor Outlet", amount: 5 },
];
const states = ["current", "partial", "stale", "missing", "failed"];
const params = new URLSearchParams(location.search);
const state = {
  scenario: states.includes(params.get("state")) ? params.get("state") : "current",
  language: params.get("lang") === "en" ? "en" : "zh",
  layer: params.get("layer") === "topology" ? "topology" : "molar",
  unit: params.get("unit") === "kmol/h" ? "kmol/h" : "mol/s",
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
const flowFormat = new Intl.NumberFormat("en", { maximumFractionDigits: 3 });
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

for (const key of ["layer", "unit", "scenario", "language"]) {
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

function displayAmount(amount) {
  const converted = state.unit === "kmol/h" ? amount * 3.6 : amount;
  return `${flowFormat.format(converted)} ${state.unit}`;
}

function color(amount, maximum) {
  if (state.layer !== "molar" || amount === null) return neutral;
  const ratio = maximum > 0 ? amount / maximum : 0;
  return `rgb(${low.map((value, index) => Math.round(value + (high[index] - value) * ratio)).join(", ")})`;
}

function render() {
  document.documentElement.lang = state.language === "zh" ? "zh-CN" : "en";
  document.title = `RadishFlow Studio · ${t("molarFlow")}`;
  for (const item of document.querySelectorAll("[data-text]")) item.textContent = t(item.dataset.text);
  for (const item of document.querySelectorAll("[data-label]")) item.setAttribute("aria-label", t(item.dataset.label));
  $("scenario").replaceChildren(...states.map((value) => new Option(t(value), value)));
  for (const key of ["layer", "unit", "scenario", "language"]) $(key).value = state[key];

  const current = ["current", "partial"].includes(state.scenario);
  const failed = state.scenario === "failed";
  const amounts = new Map(streams.map(({ id, amount }) => [
    id, current && !(state.scenario === "partial" && id === "vapor") ? amount : null,
  ]));
  const maximum = Math.max(0, ...Array.from(amounts.values()).filter((amount) => amount !== null));
  const showScale = current && state.layer === "molar";
  $("status").textContent = t(state.scenario);
  $("status").dataset.state = state.scenario;
  $("legend-title").textContent = state.layer === "molar" ? t("molarFlow") : t("topology");
  $("origin").textContent = current ? `${t("run")} #1 · r0` :
    state.scenario === "stale" ? `${t("priorRun")} #1 · r0` : failed ? `${t("run")} #2 · r1` : "";
  $("scale").hidden = !showScale;
  $("min").textContent = showScale ? displayAmount(0) : "";
  $("max").textContent = showScale ? displayAmount(maximum) : "";
  $("layer-note").textContent = t(state.layer === "topology" ? "topologyNote" :
    ({ current: "scaleNote", partial: "partialNote", stale: "staleNote", missing: "missingNote", failed: "failedNote" })[state.scenario]);
  $("issue-pin").hidden = !failed;
  $("issue-detail").hidden = !(failed && state.issueOpen);
  for (const { id, name } of streams) {
    const amount = amounts.get(id);
    const { button, swatch, value } = rows.get(id);
    const label = labels.get(id);
    const formatted = amount === null ? t("unavailable") : displayAmount(amount);
    const stroke = color(amount, maximum);
    for (const control of [label, button]) control.setAttribute("aria-pressed", String(id === state.selected));
    label.hidden = !showScale;
    label.textContent = formatted;
    label.setAttribute("aria-label", `${name} · ${amount === null ? t("noValue") : formatted}`);
    label.dataset.kind = amount === null ? "unavailable" : amount === 0 ? "zero" : "positive";
    label.dataset.stroke = stroke;
    swatch.style.backgroundColor = stroke;
    value.textContent = amount === null && current ? t("noValue") : formatted;
    value.dataset.kind = label.dataset.kind;
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
  const { width, height } = diagram;
  const ratio = devicePixelRatio || 1;
  canvas.width = Math.round(width * ratio);
  canvas.height = Math.round(height * ratio);
  ctx.setTransform(ratio, 0, 0, ratio, 0, 0);
  ctx.lineJoin = "round";
  ctx.lineCap = "round";
  const graph = geometry(diagram);
  for (const { id } of streams) {
    const { points, label } = graph[id];
    const labelButton = labels.get(id);
    const stroke = labelButton.dataset.stroke;
    if (state.selected === id) strokePath(points, "#c8dbee", 9);
    ctx.setLineDash(state.scenario === "partial" && id === "vapor" && state.layer === "molar" ? [5, 5] : []);
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

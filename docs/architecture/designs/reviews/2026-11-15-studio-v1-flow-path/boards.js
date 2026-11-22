"use strict";

const strings = {
  zh: {
    navigation: "流程导航", canvas: "流程图", results: "流程结果", path: "二维流向与路径",
    legendLabel: "画布图例", topology: "连接方向", selection: "所选流股", diagnostic: "诊断",
    stream: "所选流股", connections: "路径端点", upstream: "上游", downstream: "下游",
    outletBoundary: "流程出口", unbound: "加热器入口未连接", openEnd: "未连接",
    viewIssue: "查看运行诊断", backIssue: "← 返回流股", diagnosticTitle: "运行诊断详情",
    locateIssue: "定位 Heater 入口", locate: "定位",
    molarFlow: "摩尔流量", unit: "显示单位", resultSource: "结果来源",
    viewResult: "在流程结果中查看 →", backCanvas: "← 返回流程图",
    play: "播放方向示意", pause: "暂停方向示意",
    paused: "显示静态连接箭头。", playing: "方向示意播放中。",
    noFlow: "零流量 · 保留连接方向。", noStreamResult: "所选流股无有效流量 · 保留连接方向。",
    reduced: "已按系统设置减少动态效果。",
    current: "结果当前", stale: "结果过期", missing: "尚无结果", failed: "运行失败",
    currentOrigin: "运行 #1 · 修订 r0", staleOrigin: "输入已修改，等重跑后更新。",
    missingOrigin: "此流程尚无可用快照。", failedOrigin: "本次运行失败，当前流量不可用。",
    reviewTitle: "画板走查", scenario: "样例状态", language: "语言", largeText: "20 px 基准字号",
    zero: "当前结果 · 零流量", partial: "当前结果 · 所选流股无流量数据",
  },
  en: {
    navigation: "Flowsheet navigation", canvas: "Flowsheet", results: "Flowsheet results",
    path: "Flow direction and path", legendLabel: "Canvas legend", topology: "Connection direction",
    selection: "Selected stream", diagnostic: "Diagnostic", stream: "Selected stream",
    connections: "Path endpoints", upstream: "Upstream", downstream: "Downstream",
    outletBoundary: "Flowsheet outlet", unbound: "Unbound heater inlet", openEnd: "Unconnected",
    viewIssue: "View run diagnostic", backIssue: "← Back to stream", diagnosticTitle: "Run diagnostic",
    locateIssue: "Locate Heater inlet", locate: "Locate",
    molarFlow: "Molar flow", unit: "Display unit", resultSource: "Result source",
    viewResult: "View in flowsheet results →", backCanvas: "← Back to flowsheet",
    play: "Play direction preview", pause: "Pause direction preview",
    paused: "Showing static connection arrows.", playing: "Previewing direction.",
    noFlow: "Zero flow · connection direction stays visible.",
    noStreamResult: "No valid flow for this stream · showing connection direction.",
    reduced: "Motion is reduced by your system setting.",
    current: "Current result", stale: "Stale result", missing: "No result yet", failed: "Run failed",
    currentOrigin: "Run #1 · revision r0", staleOrigin: "Inputs changed. Rerun to update results.",
    missingOrigin: "This flowsheet has no available snapshot.",
    failedOrigin: "The run failed. Current flow is unavailable.",
    reviewTitle: "Review controls", scenario: "Scenario", language: "Language", largeText: "20 px base type",
    zero: "Current result · zero flow", partial: "Current result · stream flow unavailable",
  },
};

const units = {
  feed: { name: "Feed", kind: "feed", desktop: [10, 51], narrow: [50, 13] },
  heater: { name: "Heater", kind: "heater", desktop: [36, 51], narrow: [50, 36] },
  flash: { name: "Flash Drum", kind: "flash", desktop: [64, 51], narrow: [50, 60] },
};

const streams = {
  "stream-feed": {
    name: "Feed", source: ["feed", "outlet"], sink: ["heater", "inlet"],
    desktop: [23, 37], narrow: [72, 24], flow: 5,
  },
  "stream-heated": {
    name: "Heated Outlet", source: ["heater", "outlet"], sink: ["flash", "inlet"],
    desktop: [50, 37], narrow: [80, 48], flow: 5,
  },
  "stream-liquid": {
    name: "Liquid Outlet", source: ["flash", "liquid"], sink: null,
    desktop: [87.5, 23], narrow: [25, 86], flow: 0,
  },
  "stream-vapor": {
    name: "Vapor Outlet", source: ["flash", "vapor"], sink: null,
    desktop: [87.5, 66], narrow: [76, 86], flow: 5,
  },
};

const params = new URLSearchParams(location.search);
const scenarios = ["current", "zero", "stale", "missing", "failed", "partial"];
const reducedMotion = matchMedia("(prefers-reduced-motion: reduce)");
const state = {
  scenario: scenarios.includes(params.get("scenario")) ? params.get("scenario") : "current",
  language: params.get("lang") === "en" ? "en" : "zh",
  largeText: params.get("size") === "20",
  streamId: "stream-heated",
  playing: false,
  unit: "mol/s",
  page: "canvas",
};

if (state.scenario === "failed") state.streamId = "stream-feed";
if (state.scenario === "zero") state.streamId = "stream-liquid";

const byId = (id) => document.getElementById(id);
const t = (key) => strings[state.language][key];
const canvas = byId("diagram-lines");
const diagram = canvas.parentElement;
const diagramObjects = byId("diagram-objects");
const drawing = canvas.getContext("2d");
const nodes = new Map();
const streamTags = new Map();
let animationFrame;
let animationStart;
let resultOpener;

function position(element, desktop, narrow) {
  for (const [layout, [x, y]] of [["desktop", desktop], ["narrow", narrow]]) {
    element.style.setProperty(`--${layout}-x`, `${x}%`);
    element.style.setProperty(`--${layout}-y`, `${y}%`);
  }
}

function addDiagramObjects() {
  for (const [unitId, unit] of Object.entries(units)) {
    const node = document.createElement("div");
    node.tabIndex = -1;
    node.setAttribute("role", "group");
    node.className = "diagram-object node";
    node.dataset.unit = unitId;
    const icon = document.createElement("span");
    icon.className = "node-glyph";
    icon.dataset.kind = unit.kind;
    icon.setAttribute("aria-hidden", "true");
    const name = document.createElement("span");
    name.className = "node-name";
    name.textContent = unit.name;
    const mark = document.createElement("span");
    mark.className = "warning-swatch";
    mark.textContent = "!";
    mark.tabIndex = -1;
    mark.setAttribute("role", "img");
    mark.hidden = true;
    node.append(icon, name, mark);
    position(node, unit.desktop, unit.narrow);
    diagramObjects.append(node);
    nodes.set(unitId, node);
  }

  for (const [streamId, stream] of Object.entries(streams)) {
    const tag = document.createElement("button");
    tag.type = "button";
    tag.className = "diagram-object stream-tag";
    tag.dataset.stream = streamId;
    tag.textContent = stream.name;
    tag.addEventListener("click", () => selectStream(streamId));
    position(tag, stream.desktop, stream.narrow);
    diagramObjects.append(tag);
    streamTags.set(streamId, tag);
    byId("stream").add(new Option(stream.name, streamId));
  }
}

function resultStatus() {
  return state.scenario === "zero" || state.scenario === "partial" ? "current" : state.scenario;
}

function flow() {
  if (resultStatus() !== "current" || state.scenario === "partial") return null;
  return streams[state.streamId].flow;
}

function flowText() {
  const amount = flow();
  if (amount === null) return null;
  const value = state.unit === "kmol/h" ? amount * 3.6 : amount;
  return new Intl.NumberFormat(document.documentElement.lang, { maximumSignificantDigits: 6 }).format(value);
}

function issueAffectsStream() {
  return state.scenario === "failed" && state.streamId === "stream-feed";
}

function motionReason() {
  const amount = flow();
  if (resultStatus() !== "current") return `${t(resultStatus())} · ${t("topology")}`;
  if (amount === null) return t("noStreamResult");
  if (amount === 0) return t("noFlow");
  if (reducedMotion.matches) return t("reduced");
  return t(state.playing ? "playing" : "paused");
}

function canPlay() {
  const amount = flow();
  return !reducedMotion.matches && !issueAffectsStream() && Number.isFinite(amount) && amount > 0;
}

function setStatus(element) {
  const status = resultStatus();
  element.dataset.state = status;
  element.textContent = t(status);
}

function renderEndpoints(stream) {
  const endpoints = byId("endpoints");
  endpoints.replaceChildren();
  for (const [side, endpoint] of [["upstream", stream.source], ["downstream", stream.sink]]) {
    const row = document.createElement("li");
    row.className = "endpoint";
    const text = document.createElement("div");
    const direction = document.createElement("small");
    direction.textContent = t(side);
    const title = document.createElement("strong");
    const unbound = side === "downstream" && issueAffectsStream();
    title.textContent = unbound ? t("openEnd") : t("outletBoundary");
    if (endpoint && !unbound) {
      const [unit, port] = endpoint;
      title.textContent = `${units[unit].name} · ${port}`;
      const locate = document.createElement("button");
      locate.type = "button";
      locate.textContent = t("locate");
      locate.setAttribute("aria-label", `${t("locate")} ${title.textContent}`);
      locate.addEventListener("click", () => nodes.get(unit).focus());
      row.append(text, locate);
    } else {
      row.append(text);
    }
    text.append(direction, title);
    endpoints.append(row);
  }
}

function renderMotion() {
  const control = byId("motion");
  control.dataset.playing = state.playing;
  control.setAttribute("aria-disabled", String(!canPlay()));
  byId("motion-icon").textContent = state.playing ? "Ⅱ" : "▷";
  byId("motion-text").textContent = t(state.playing ? "pause" : "play");
  byId("motion-reason").textContent = motionReason();
  stopFrames();
  drawDiagram();
  if (state.playing && canPlay() && state.page === "canvas" && !document.hidden) {
    animationFrame = requestAnimationFrame(drawFrame);
  }
}

function renderSelection() {
  const stream = streams[state.streamId];
  byId("stream").value = state.streamId;
  byId("stream-id").textContent = state.streamId;
  for (const [streamId, tag] of streamTags) {
    tag.setAttribute("aria-pressed", String(streamId === state.streamId));
  }
  for (const [unitId, node] of nodes) {
    const issue = state.scenario === "failed" && unitId === "heater";
    const isSource = unitId === stream.source[0];
    const isSink = unitId === stream.sink?.[0] && !issueAffectsStream();
    node.dataset.endpoint = isSource || isSink;
    node.dataset.issue = issue;
    const marker = node.querySelector(".warning-swatch");
    marker.hidden = !issue;
    marker.setAttribute("aria-label", `Heater · inlet · ${t("unbound")}`);
    node.setAttribute("aria-label", units[unitId].name);
  }
  renderEndpoints(stream);
  byId("issue").hidden = !issueAffectsStream();
  renderReadout();
  renderMotion();
}

function renderReadout() {
  const value = flowText();
  const status = resultStatus();
  const origin = status === "current" && value === null ? t("noStreamResult") : t(`${status}Origin`);
  byId("value-block").hidden = value === null;
  byId("unit").hidden = value === null;
  byId("flow-value").textContent = value ?? "";
  byId("flow-unit").textContent = state.unit;
  byId("result-origin").textContent = origin;
  byId("show-result").hidden = value === null;
  byId("result-stream").textContent = streams[state.streamId].name;
  byId("result-flow").textContent = value === null ? origin : `${value} ${state.unit}`;
  byId("result-run").textContent = status === "current" && value !== null ? t("currentOrigin") : t(status);
  setStatus(byId("run-status"));
  setStatus(byId("results-status"));
}

function renderLanguage() {
  document.documentElement.lang = state.language === "zh" ? "zh-CN" : "en";
  for (const element of document.querySelectorAll("[data-text]")) element.textContent = t(element.dataset.text);
  for (const element of document.querySelectorAll("[data-label]")) {
    element.setAttribute("aria-label", t(element.dataset.label));
  }
  for (const option of byId("scenario").options) option.textContent = t(option.value);
  renderSelection();
}

function selectStream(streamId) {
  if (streamId !== state.streamId) state.playing = false;
  state.streamId = streamId;
  renderSelection();
}

function showPage(page, focus) {
  state.page = page;
  byId("workspace").hidden = page !== "canvas";
  byId("result-page").hidden = page !== "results";
  for (const [name, id] of [["canvas", "canvas-nav"], ["results", "result-nav"]]) {
    if (name === page) byId(id).setAttribute("aria-current", "page");
    else byId(id).removeAttribute("aria-current");
  }
  if (page === "canvas") resizeDiagram();
  renderMotion();
  focus?.focus();
}

function showIssue(show, focus) {
  byId("stream-detail").hidden = show;
  byId("diagnostic-detail").hidden = !show;
  focus?.focus();
}

function stopFrames() {
  if (animationFrame !== undefined) cancelAnimationFrame(animationFrame);
  animationFrame = undefined;
  animationStart = undefined;
}

function drawFrame(now) {
  animationStart ??= now;
  drawDiagram(((now - animationStart) / 1700) % 1);
  animationFrame = requestAnimationFrame(drawFrame);
}

function resizeDiagram() {
  const { width, height } = canvas.getBoundingClientRect();
  if (width === 0 || height === 0) return;
  diagram.classList.toggle("narrow", width < 585);
  const resolution = devicePixelRatio || 1;
  canvas.width = Math.round(width * resolution);
  canvas.height = Math.round(height * resolution);
  drawDiagram();
}

function portAt(node, side, canvasRect, crossOffset = 0) {
  const { left, top, right, bottom, width, height } = node.getBoundingClientRect();
  const centerX = left + width / 2 - canvasRect.left;
  const centerY = top + height / 2 - canvasRect.top;
  if (side === "left") return [left - canvasRect.left, centerY];
  if (side === "right") return [right - canvasRect.left, centerY + crossOffset];
  if (side === "top") return [centerX, top - canvasRect.top];
  return [centerX + crossOffset, bottom - canvasRect.top];
}

function wireFor(streamId, rect, narrow) {
  const stream = streams[streamId];
  const exitSide = narrow ? "bottom" : "right";
  const inletSide = narrow ? "top" : "left";
  const node = nodes.get(stream.source[0]);
  const source = portAt(node, exitSide, rect, stream.sink ? 0 : streamId === "stream-liquid" ? -13 : 13);

  if (stream.sink) {
    const sink = portAt(nodes.get(stream.sink[0]), inletSide, rect);
    if (streamId === "stream-feed" && state.scenario === "failed") {
      return [source, [source[0] + (sink[0] - source[0]) * .45, source[1] + (sink[1] - source[1]) * .45]];
    }
    return [source, sink];
  }

  if (narrow) {
    const x = rect.width * (streamId === "stream-liquid" ? .25 : .76);
    const y = rect.height * .75;
    return [source, [source[0], y], [x, y], [x, rect.height * .95]];
  }

  const elbowX = rect.width * (streamId === "stream-liquid" ? .765 : .79);
  const y = rect.height * (streamId === "stream-liquid" ? .31 : .74);
  return [source, [elbowX, source[1]], [elbowX, y], [rect.width * .95, y]];
}

function arrow(x, y, angle, color) {
  drawing.save();
  drawing.translate(x, y);
  drawing.rotate(angle);
  drawing.fillStyle = color;
  drawing.beginPath();
  drawing.moveTo(0, 0);
  drawing.lineTo(-10, -5.5);
  drawing.lineTo(-10, 5.5);
  drawing.closePath();
  drawing.fill();
  drawing.restore();
}

function pointAlongWire(points, phase) {
  const lengths = points.slice(1).map(([x, y], i) => Math.hypot(x - points[i][0], y - points[i][1]));
  let distance = lengths.reduce((sum, length) => sum + length, 0) * phase;
  for (const [i, length] of lengths.entries()) {
    if (distance <= length && length > 0) {
      const [x, y] = points[i];
      const [nextX, nextY] = points[i + 1];
      return [x + (nextX - x) * distance / length, y + (nextY - y) * distance / length];
    }
    distance -= length;
  }
  return points.at(-1);
}

function drawWire(points, selected, dangling, motion) {
  const color = selected ? "#1758a6" : "#527486";
  drawing.save();
  drawing.strokeStyle = color;
  drawing.lineWidth = selected ? 3.5 : 2;
  drawing.lineJoin = "round";
  drawing.beginPath();
  drawing.moveTo(...points[0]);
  for (const point of points.slice(1)) drawing.lineTo(...point);
  drawing.stroke();

  const end = points.at(-1);
  const prev = points.at(-2);
  if (dangling) {
    drawing.fillStyle = "#f8fbfd";
    drawing.beginPath();
    drawing.arc(...end, 4, 0, Math.PI * 2);
    drawing.fill();
    drawing.stroke();
  } else {
    arrow(...end, Math.atan2(end[1] - prev[1], end[0] - prev[0]), color);
  }
  if (motion !== undefined) {
    drawing.fillStyle = "#fff";
    for (const phase of [motion, (motion + .5) % 1]) {
      drawing.beginPath();
      drawing.arc(...pointAlongWire(points, phase), 3.5, 0, Math.PI * 2);
      drawing.fill();
      drawing.stroke();
    }
  }
  drawing.restore();
}

function drawDiagram(motion) {
  const rect = canvas.getBoundingClientRect();
  if (rect.width === 0 || rect.height === 0) return;
  drawing.setTransform(canvas.width / rect.width, 0, 0, canvas.height / rect.height, 0, 0);
  drawing.clearRect(0, 0, rect.width, rect.height);
  const narrow = diagram.classList.contains("narrow");
  for (const streamId of Object.keys(streams)) {
    const selected = streamId === state.streamId;
    const dangling = state.scenario === "failed" && streamId === "stream-feed";
    const points = wireFor(streamId, rect, narrow);
    drawWire(points, selected, dangling, selected ? motion : undefined);
  }
}

addDiagramObjects();
for (const scenario of scenarios) byId("scenario").add(new Option(t(scenario), scenario));
byId("scenario").value = state.scenario;
byId("language").value = state.language;
byId("large-text").checked = state.largeText;
document.documentElement.dataset.size = state.largeText ? "20" : "14";
byId("review-controls").hidden = params.get("review") !== "1";

byId("stream").addEventListener("change", (event) => selectStream(event.target.value));
byId("unit").addEventListener("change", (event) => {
  state.unit = event.target.value;
  renderReadout();
});
byId("motion").addEventListener("click", () => {
  if (!canPlay()) return;
  state.playing = !state.playing;
  renderMotion();
});
byId("inspect-issue").addEventListener("click", () => showIssue(true, byId("back-issue")));
byId("back-issue").addEventListener("click", () => showIssue(false, byId("inspect-issue")));
byId("locate-issue").addEventListener("click", () => nodes.get("heater").querySelector(".warning-swatch").focus());
for (const id of ["show-result", "result-nav"]) {
  byId(id).addEventListener("click", (event) => {
    if (state.page === "results") return;
    resultOpener = event.currentTarget;
    showPage("results", byId("results-title"));
  });
}
byId("canvas-nav").addEventListener("click", () => {
  if (state.page !== "canvas") showPage("canvas", byId("canvas-title"));
});
byId("back-canvas").addEventListener("click", () => {
  showPage("canvas", resultOpener?.hidden ? byId("canvas-title") : resultOpener);
});

byId("scenario").addEventListener("change", (event) => {
  state.scenario = event.target.value;
  state.playing = false;
  if (state.scenario === "failed") state.streamId = "stream-feed";
  if (state.scenario === "zero") state.streamId = "stream-liquid";
  if (!issueAffectsStream()) showIssue(false);
  renderSelection();
});
byId("language").addEventListener("change", (event) => {
  state.language = event.target.value;
  renderLanguage();
});
byId("large-text").addEventListener("change", (event) => {
  document.documentElement.dataset.size = event.target.checked ? "20" : "14";
  resizeDiagram();
});
reducedMotion.addEventListener("change", () => {
  if (reducedMotion.matches) state.playing = false;
  renderMotion();
});
document.addEventListener("visibilitychange", renderMotion);
new ResizeObserver(resizeDiagram).observe(diagram);
document.fonts.ready.then(resizeDiagram);
renderLanguage();
resizeDiagram();

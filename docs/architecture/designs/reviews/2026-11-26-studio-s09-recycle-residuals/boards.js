"use strict";

const byId = id => document.getElementById(id);
const scene = byId("scene");
const language = byId("language");
const context = byId("context");
const checkpoint = byId("checkpoint");
const primary = byId("primary");
const source = byId("source");
let destination = "review";
let generation = 0;

const tr = (zh, en) => language.value === "zh" ? zh : en;
const currentReport = () => reports[["success", "running90"].includes(scene.value) ? "090" : "099"];
const isRunning = () => scene.value.startsWith("running");
const phaseName = (phase, index) => (phase === "verification"
  ? tr("独立复核", "Verification") : tr("迭代", "Iteration")) + " " + index;
function amount(n) {
  const rounded = Number(n.toPrecision(6));
  const magnitude = Math.abs(rounded);
  return magnitude !== 0 && (magnitude < 1e-4 || magnitude >= 1e6)
    ? rounded.toExponential().replace("e+", "e") : String(rounded);
}
const signed = (n, format) => (n > 0 ? "+" : "") + format(n);

function el(tag, content, cls) {
  const element = document.createElement(tag);
  if (content !== undefined) element.textContent = content;
  if (cls) element.className = cls;
  return element;
}

function pair(label, value, numeric = false) {
  const group = el("div");
  group.append(el("dt", label), el("dd", value, numeric ? "num" : ""));
  return group;
}

function disclosure(key, label) {
  const details = el("details");
  details.dataset.key = key;
  details.append(el("summary", label));
  return details;
}

function panel(title, cls = "") {
  const card = el("section", undefined, "card " + cls);
  card.append(el("h2", title));
  return card;
}

function setHeading(title, status, tone, path) {
  byId("heading").textContent = title;
  byId("badge").className = "badge " + tone;
  byId("badge").textContent = status;
  byId("path").textContent = path || tr("流程运行  /  稳态运行计划  /  流程结果", "Flow run  /  Steady-state plan  /  Flow results");
}

function showSource(r) {
  const info = el("dl");
  info.append(
    pair(tr("回流分率", "Recycle fraction"), r.fraction.toFixed(2), true),
    pair(tr("依赖块", "Dependency block"), String(r.block) + " · " + r.members.join(" + ")),
    pair(tr("撕裂流股", "Tear stream"), r.tear.stream_id, true),
    pair(tr("运行来源", "Run source"), tr("请求步骤 ", "Request step ") + r.step),
    pair(tr("输入修订", "Input revision"), String(r.revision)),
    pair(tr("方法", "Method"), r.method, true)
  );
  const binding = disclosure("binding", tr("端口与输入身份", "Ports and input identity"));
  const ports = el("dl");
  ports.append(pair(tr("源端口", "Source port"), r.tear.source.unit_id + "." + r.tear.source.port, true),
    pair(tr("汇端口", "Sink port"), r.tear.sink.unit_id + "." + r.tear.sink.port, true),
    pair(tr("工程", "Project"), r.documentId, true), pair("SHA-256", r.fileSha256, true));
  binding.append(ports);
  source.replaceChildren(el("h2", tr("回流块", "Recycle block") + " " + r.block), info, binding);
}

function checks(r) {
  const details = disclosure("checks", tr("其他检查", "Other checks"));
  const list = el("dl");
  const t = r.tolerances;
  list.append(pair(tr("总流量容差", "Total flow tolerance"), amount(t.total_absolute_flow_mol_s) + " mol/s", true),
    pair(tr("温度容差", "Temperature tolerance"), amount(t.temperature_k) + " K", true),
    pair(tr("压力容差", "Pressure tolerance"), amount(t.pressure_pa) + " Pa", true));
  details.append(list);
  return details;
}

function metric(label, value, limit, unit) {
  const cell = el("div", undefined, "metric");
  const pass = value <= limit;
  cell.append(el("small", label), el("strong", amount(value) + " " + unit, "reading num"),
    el("small", tr("容差 ≤ ", "Tolerance ≤ ") + amount(limit) + " " + unit),
    el("span", pass ? tr("达标", "Pass") : tr("超限", "Over limit"), "verdict " + (pass ? "good" : "over")));
  return cell;
}

function sampleRow(r, sample, meaning) {
  const [phase, n, absolute, scaled] = sample;
  const row = el("div", undefined, "sample");
  const stage = el("div", undefined, "stage");
  stage.append(el("strong", phaseName(phase, n)));
  if (meaning) stage.append(el("small", meaning));
  row.append(stage,
    metric(tr("最大绝对残差", "Max absolute residual"), absolute, r.tolerances.component_absolute_flow_mol_s, "mol/s"),
    metric(tr("最大缩放残差", "Max scaled residual"), scaled, r.tolerances.scaled_flow, tr("无量纲", "unitless")));
  return row;
}

function components(r, last) {
  const details = disclosure("components", tr("末次组分明细", "Final component details") + " · " + phaseName(last[0], last[1]));
  details.className = "card";
  for (const [id, c] of Object.entries(r.components)) {
    const part = el("section", undefined, "component");
    part.append(el("h3", id === "ethane" ? tr("乙烷 · ethane", "Ethane") : tr("甲烷 · methane", "Methane")));
    const quantities = format => {
      const list = el("dl");
      list.append(pair(tr("Mixer 假设入口", "Assumed mixer inlet"), format(c.assumed) + " mol/s", true),
        pair(tr("Splitter 返回出口", "Returned splitter outlet"), format(c.returned) + " mol/s", true),
        pair(tr("返回减假设", "Return − assumption"), signed(c.residual, format) + " mol/s", true),
        pair(tr("尺度", "Scale"), format(c.scale) + " mol/s", true),
        pair(tr("缩放绝对值", "Scaled magnitude"), format(Math.abs(c.residual) / c.scale) + " " + tr("无量纲", "unitless"), true));
      return list;
    };
    const exact = disclosure("exact-" + id, tr("展开原始 SI 精度", "Raw SI precision"));
    exact.className = "exact";
    exact.append(quantities(String));
    part.append(quantities(amount), exact);
    details.append(part);
  }
  return details;
}

function history(r) {
  const details = disclosure("history", tr("完整残差轨迹", "Full residual trace") + " · " + r.trace.length);
  details.className = "card history";
  const list = el("div", undefined, "trace-list");
  list.tabIndex = 0;
  list.setAttribute("role", "region");
  list.setAttribute("aria-label", tr("按求值顺序排列的残差", "Residuals in evaluation order"));
  details.append(el("p", tr("历史样本仅保留两项最大值，逐组分明细未采集。", "Historical samples retain both maxima. Component details were not collected."), "note"), list);
  details.addEventListener("toggle", () => {
    if (details.open && !list.childElementCount) {
      list.replaceChildren(...r.trace.map(sample => sampleRow(r, sample)));
    }
  });
  return details;
}

function routeKey(r) {
  return [r.fileSha256, r.step, r.revision, generation].join("/");
}

function sourceAvailable() {
  return context.value === "original" || (context.value === "pending" && currentReport().status === "current");
}

function routeAction(r, next, label) {
  const button = el("button", label, "action");
  button.id = "route-action";
  const key = routeKey(r);
  button.onclick = () => {
    if (!sourceAvailable() || key !== routeKey(currentReport()) || (next === "stream" && r.status !== "current")) return;
    destination = next;
    render();
    byId("heading").focus();
  };
  return button;
}

function showTerminal(r) {
  const success = r.status === "current";
  setHeading(success ? tr("回流块详情", "Recycle block details") : tr("回流失败诊断", "Recycle failure diagnostics"),
    success ? tr("当前结果 · Current", "Current") : tr("运行失败 · Failed", "Failed"), success ? "good" : "over");
  const last = r.trace.at(-1);
  if (!success) {
    const failure = panel(tr("迭代预算耗尽", "Iteration budget exhausted"), "failure");
    failure.append(el("p", tr("最后试算 ", "Last trial ") + last[1] + " · " + tr("独立复核未执行", "Verification not executed")),
      el("p", tr("本次无正式结果", "No qualified result for this run")),
      el("p", r.diagnostic, "num note"));
    primary.append(failure);
  }
  const summary = panel(tr("收敛判据", "Convergence criteria"));
  const head = el("div", undefined, "section-heading");
  head.append(summary.firstChild, el("span", r.iterations + tr(" 次迭代", " iterations") + " · " + r.verifications + tr(" 次复核", " verifications"), "muted"));
  summary.append(head);
  if (success) {
    const candidate = r.trace.findIndex(([phase, , a, s]) => phase === "iteration"
      && a <= r.tolerances.component_absolute_flow_mol_s && s <= r.tolerances.scaled_flow);
    if (candidate > 0) summary.append(sampleRow(r, r.trace[candidate - 1], tr("候选前一次", "Before candidate")));
    if (candidate >= 0) summary.append(sampleRow(r, r.trace[candidate], tr("首次同时达标 · 候选", "First joint pass · candidate")));
    summary.append(sampleRow(r, last, tr("复核通过", "Verified")));
  } else {
    summary.append(sampleRow(r, last, tr("最后试算", "Last trial")));
  }
  primary.append(summary);
  const result = panel(success ? tr("正式流股", "Qualified stream") : tr("诊断定位", "Diagnostic target"), success ? "result" : "");
  const row = el("div", undefined, "action-row");
  if (success) {
    row.append(el("span", tr("生产端复核返回值", "Verified source return") + " · " + amount(r.formalStream.flow) + " mol/s"),
      routeAction(r, "stream", tr("查看正式流股", "View qualified stream") + "  " + r.tear.stream_id));
  } else {
    row.append(el("span", r.members.join(" + "), "num"),
      routeAction(r, "object", tr("定位撕裂流股", "Locate tear stream") + "  " + r.tear.stream_id));
  }
  result.append(row);
  primary.append(result, components(r, last), history(r));
  source.append(checks(r));
}

function showTarget(r) {
  const formal = destination === "stream";
  setHeading(r.tear.stream_id, formal ? tr("当前结果 · Current", "Current") : tr("工程对象", "Project object"),
    formal ? "good" : "neutral", formal ? tr("流程结果  /  正式流股", "Flow results  /  Qualified stream") : tr("流程图  /  流股检查器", "Flowsheet  /  Stream inspector"));
  const card = panel(formal ? tr("生产端复核返回值", "Verified source return") : tr("撕裂流股连接", "Tear stream binding"));
  const facts = el("dl");
  if (formal) {
    facts.append(pair(tr("总摩尔流量", "Total molar flow"), String(r.formalStream.flow) + " mol/s", true));
    for (const [id, value] of Object.entries(r.formalStream.components)) facts.append(pair(id, String(value) + " mol/s", true));
  } else {
    facts.append(pair(tr("源端口", "Source port"), r.tear.source.unit_id + "." + r.tear.source.port, true),
      pair(tr("汇端口", "Sink port"), r.tear.sink.unit_id + "." + r.tear.sink.port, true));
  }
  const back = el("button", tr("返回回流详情", "Back to recycle details"));
  back.id = "back";
  back.onclick = () => { destination = "review"; render(); byId("route-action").focus(); };
  card.append(facts, back);
  primary.append(card);
}

function updateCheckpoints() {
  const r = currentReport();
  const options = r.status === "current"
    ? [["first", tr("迭代 1 开始", "Iteration 1 starts")], ["candidate", tr("迭代 229 开始", "Iteration 229 starts")],
       ["verify", tr("复核 1 开始", "Verification 1 starts")], ["verified", tr("复核 1 完成", "Verification 1 ends")]]
    : [["first", tr("迭代 1 开始", "Iteration 1 starts")], ["limit", tr("迭代 500 开始", "Iteration 500 starts")],
       ["exhausted", tr("迭代 500 完成", "Iteration 500 ends")]];
  const previous = checkpoint.value;
  checkpoint.replaceChildren(...options.map(([value, label]) => new Option(label, value)));
  checkpoint.value = options.some(([v]) => v === previous) ? previous : (r.status === "current" ? "verify" : "limit");
}

function showRunning(r) {
  setHeading(tr("稳态运行计划", "Steady-state plan"), tr("已应用计划", "Applied plan"), "neutral",
    tr("流程运行  /  稳态运行计划", "Flow run  /  Steady-state plan"));
  const last = r.trace.at(-1);
  const positions = {
    first: ["iteration", 1, null, false],
    candidate: ["iteration", r.iterations, r.trace.at(-3), false],
    verify: ["verification", 1, r.trace.at(-2), false],
    verified: ["verification", 1, last, true],
    limit: ["iteration", r.iterations, r.trace.at(-2), false],
    exhausted: ["iteration", r.iterations, last, true]
  };
  const [phase, n, sample, completed] = positions[checkpoint.value];
  const activity = panel(tr("运行中", "Running"), "activity");
  const current = el("section");
  current.append(el("h3", completed ? tr("刚完成的执行阶段", "Execution stage just completed") : tr("正在执行", "Executing now")),
    el("p", tr("块 ", "Block ") + r.block + " · " + phaseName(phase, n)));
  const recent = el("section");
  recent.append(el("h3", tr("最近完成的求值", "Last completed evaluation")));
  if (sample) {
    recent.append(el("p", phaseName(sample[0], sample[1])));
    const list = el("dl", undefined, "readouts");
    list.append(pair(tr("最大绝对残差", "Max absolute residual"), amount(sample[2]) + " mol/s", true),
      pair(tr("最大缩放残差", "Max scaled residual"), amount(sample[3]) + " " + tr("无量纲", "unitless"), true));
    recent.append(list);
  } else {
    recent.append(el("p", tr("尚无已完成求值", "No evaluation has completed yet")));
  }
  activity.append(current, recent);
  byId("activity-bar").append(activity);
  const boundary = panel(tr("固定温压边界", "Fixed-temperature and pressure boundary"));
  const conditions = el("dl", undefined, "readouts");
  conditions.append(pair(tr("温度", "Temperature"), r.temperature + " K", true),
    pair(tr("压力", "Pressure"), r.pressure + " Pa", true));
  boundary.append(conditions);
  primary.append(boundary);
}

function render(preserve = false) {
  const open = preserve ? [...document.querySelectorAll("details[open]")].map(d => d.dataset.key) : [];
  document.documentElement.lang = language.value === "zh" ? "zh-CN" : "en";
  byId("controls").setAttribute("aria-label", tr("评审设置", "Review settings"));
  document.querySelectorAll("[data-en]").forEach(n => {
    if (!n.dataset.zh) n.dataset.zh = n.textContent;
    n.textContent = tr(n.dataset.zh, n.dataset.en);
  });
  byId("checkpoint-label").hidden = !isRunning();
  byId("activity-bar").replaceChildren();
  byId("activity-bar").hidden = !isRunning();
  byId("context-label").hidden = isRunning();
  context.querySelector('[value="pending"]').disabled = currentReport().status !== "current";
  if (isRunning()) updateCheckpoints();
  const r = currentReport();
  primary.replaceChildren();
  source.hidden = !isRunning() && !sourceAvailable();
  if (source.hidden) {
    source.replaceChildren();
    const message = {
      edited: tr("输入已修改，请按新输入重跑。", "Input changed. Rerun with the new input."),
      switched: tr("已切换工程，原工程的运行详情已关闭。", "Project switched. The previous run details are closed."),
      rerun: tr("重跑回执已接收，原运行的详情入口已失效。", "A new run receipt was accepted. The original run details have expired.")
    };
    setHeading(tr("流程结果", "Flow results"), context.value === "edited" && r.status === "current" ? "Stale" : tr("原来源已失效", "Original source expired"), "neutral");
    primary.append(panel(message[context.value]));
    return;
  }
  showSource(r);
  if (!isRunning() && context.value === "pending") {
    source.append(el("p", tr("重跑尚未返回，当前读数仍归原运行。", "The rerun is pending. These readings still belong to the original run."), "note"));
  }
  if (isRunning()) showRunning(r);
  else if (destination === "review") showTerminal(r);
  else showTarget(r);
  for (const details of document.querySelectorAll("details")) details.open = open.includes(details.dataset.key);
}

scene.onchange = () => { generation++; context.value = "original"; checkpoint.replaceChildren(); destination = "review"; render(); };
context.onchange = () => {
  if (!sourceAvailable()) { generation++; destination = "review"; }
  render();
};
checkpoint.onchange = () => render();
language.onchange = () => render(true);
byId("size").onchange = e => { document.documentElement.style.fontSize = e.target.value + "px"; };
render();

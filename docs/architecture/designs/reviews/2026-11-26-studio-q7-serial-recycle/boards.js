"use strict";

const byId = id => document.getElementById(id);
const scene = byId("scene");
const context = byId("context");
const language = byId("language");
const review = byId("review");
const destination = byId("destination");
const tr = (zh, en) => language.value === "en" ? en : zh;
const run = () => q7Runs[scene.value];
const blocks = () => [q7Blocks.upstream, q7Blocks[scene.value]];
const current = () => run().status === "current";
const streamId = block => block.tear.stream_id;
const blockName = block => tr("第 " + block.number + " 块", "Block " + block.number);
const runName = () => tr("运行 #", "Run #") + run().run;
let generation = 0;
let visit = null;

function node(tag, content, className) {
  const element = document.createElement(tag);
  if (content !== undefined) element.textContent = content;
  if (className) element.className = className;
  return element;
}

function facts(entries) {
  const list = node("dl", undefined, "facts");
  for (const [name, value] of entries) list.append(node("dt", name), node("dd", value));
  return list;
}

function disclosure(label, kind) {
  const details = node("details");
  details.dataset.kind = kind;
  details.append(node("summary", label));
  return details;
}

function amount(value) {
  const rounded = Number(value.toPrecision(6));
  return rounded !== 0 && Math.abs(rounded) < 1e-4 ? rounded.toExponential() : String(rounded);
}

function phase(sample) {
  return (sample[0] === "verification" ? tr("独立复核 ", "Verification ") : tr("试算 ", "Iteration ")) + sample[1];
}

function bindings(block) {
  const port = endpoint => endpoint.unit_id + "." + endpoint.port;
  return [[tr("生产端", "Source port"), port(block.tear.source)], [tr("消费端", "Sink port"), port(block.tear.sink)]];
}

function sourceIdentity() {
  const report = run();
  const details = disclosure(tr("运行来源", "Run source"), "source");
  details.append(facts([
    [tr("工程", "Project"), report.project],
    [tr("方法", "Method"), report.method],
    [tr("块间传递量 · s-bleed", "Block transfer · s-bleed"), report.bleed + " mol/s"],
    [tr("计划配置 SHA-256", "Plan configuration SHA-256"), report.planHash],
    [tr("输入文件 SHA-256", "Input file SHA-256"), report.fileHash],
    [tr("对照用 CLI 步骤", "Reference CLI step"), report.step],
    ...(report.diagnostic ? [[tr("诊断码", "Diagnostic code"), report.diagnostic]] : [])
  ]));
  return details;
}

function receipt() {
  const report = run();
  const box = node("section", undefined, "receipt" + (current() ? "" : " failed"));
  const heading = node("div", undefined, "status-line");
  heading.append(node("h2", tr("整图结果", "Flowsheet result")),
    node("strong", current() ? "Current" : "Failed", "tag " + (current() ? "pass" : "fail")),
    node("strong", current()
      ? tr(report.streamCount + " 股正式结果已发布", report.streamCount + " stream results published")
      : tr("第 5 块迭代预算耗尽 · 本次无正式结果", "Block 5 exhausted its iteration budget · No published results for this run")));
  const identity = node("div", undefined, "identity");
  identity.append(node("span", runName()), node("span", tr("输入修订 ", "Input revision ") + report.revision),
    node("span", report.heaterCalls
      ? tr("下游 heater-1 已执行 · " + report.heaterCalls + " 次", "Downstream heater-1 ran · " + report.heaterCalls + " call")
      : tr("下游 heater-1 未执行", "Downstream heater-1 did not run")));
  box.append(heading, identity, sourceIdentity());
  return box;
}

function sampleRow(block, sample, last = false) {
  const row = node("div", undefined, "metrics");
  const stage = node("div");
  stage.append(node("small", last ? tr("末次求值", "Final evaluation") : tr("求值", "Evaluation")), node("strong", phase(sample)));
  row.append(stage);
  const t = block.tolerances;
  for (const [label, value, limit, unit] of [
    [tr("最大绝对残差", "Max absolute residual"), sample[2], t.component_absolute_flow_mol_s, "mol/s"],
    [tr("最大缩放残差", "Max scaled residual"), sample[3], t.scaled_flow, tr("无量纲", "unitless")]
  ]) {
    const cell = node("div");
    const within = value <= limit;
    const reading = node("strong", amount(value) + " " + unit, "number" + (within ? "" : " exceed"));
    reading.title = String(value) + " " + unit;
    cell.append(node("small", label), reading,
      node("small", (within ? tr("达标", "Pass") : tr("超限", "Over limit")) + tr(" · 容差 ≤ ", " · Limit ≤ ") + amount(limit) + " " + unit));
    row.append(cell);
  }
  return row;
}

function history(block) {
  const details = disclosure(tr("完整残差轨迹", "Full residual trace") + " · " + block.trace.length, "trace");
  const list = node("div", undefined, "trace");
  list.tabIndex = 0;
  list.setAttribute("role", "region");
  list.setAttribute("aria-label", blockName(block) + " · " + tr("按求值顺序排列的残差", "Residuals in evaluation order"));
  details.append(node("p", tr("各次求值记录两项最大残差，末次另有组分明细。",
    "Each evaluation records both maxima. Component details are available for the final evaluation."), "note"), list);
  details.addEventListener("toggle", () => {
    if (details.open && !list.childElementCount) list.append(...block.trace.map(sample => sampleRow(block, sample)));
  });
  return details;
}

function components(block) {
  const details = disclosure(tr("末次组分与判据", "Final components and criteria"), "components");
  for (const c of block.components) {
    const section = node("div", undefined, "component");
    section.append(node("h3", c.id));
    const quantities = format => facts([
      [tr("Mixer 假设入口", "Assumed mixer inlet"), format(c.assumed) + " mol/s"],
      [tr("Splitter 返回出口", "Returned splitter outlet"), format(c.returned) + " mol/s"],
      [tr("返回减假设", "Return − assumption"), format(c.residual) + " mol/s"],
      [tr("组分尺度", "Component scale"), format(c.scale) + " mol/s"],
      [tr("缩放绝对值", "Scaled magnitude"), format(Math.abs(c.residual) / c.scale) + " " + tr("无量纲", "unitless")]
    ]);
    const exact = disclosure(tr("原始 SI 精度", "Raw SI precision"), "precision");
    exact.append(quantities(String));
    section.append(quantities(amount), exact);
    details.append(section);
  }
  const t = block.tolerances;
  details.append(facts([
    [tr("总摩尔流量容差", "Total molar flow tolerance"), t.total_absolute_flow_mol_s + " mol/s"],
    [tr("温度容差", "Temperature tolerance"), t.temperature_k + " K"],
    [tr("压力容差", "Pressure tolerance"), t.pressure_pa + " Pa"]
  ]));
  return details;
}

// A visit belongs to the displayed receipt and open project instance, including when stream IDs match.
function eligible(link) {
  if (!link || context.value !== "original" || link.generation !== generation || link.report !== run()) return false;
  const block = blocks().find(item => item.number === link.number && streamId(item) === link.stream);
  return Boolean(block && (current() ? run().formal : run().objects)[link.stream]);
}

function blockPanel(block) {
  const panel = node("section", undefined, "block");
  panel.dataset.block = block.number;
  const top = node("div", undefined, "block-top");
  const identity = node("div");
  const title = node("h3", undefined, "block-title");
  title.append(node("span", blockName(block)), node("code", streamId(block)));
  identity.append(title, node("p", block.members.join(" + "), "note"));
  const status = node("div");
  const verified = block.trace.at(-1)[0] === "verification" && block.verifications > 0;
  const verdict = verified
    ? (current() ? tr("本块复核通过", "Block verification passed") : tr("本块复核通过 · 仅诊断", "Block verification passed · Diagnostic only"))
    : tr("复核未执行", "Verification not run");
  status.append(node("strong", verdict, "tag " + (verified ? "pass" : "fail")),
    node("p", tr("迭代 " + block.iterations + " 次 · 复核 " + block.verifications + " 次",
      "Iterations " + block.iterations + " · Verifications " + block.verifications), "note"));
  const actions = node("div", undefined, "action-line");
  const button = node("button", current() ? tr("查看正式流股 ", "View result stream ") : tr("定位工程流股 ", "Locate project stream "));
  button.append(node("span", streamId(block) + " →", "stream-id"));
  button.id = "locate-" + block.number;
  const link = {report: run(), generation, stream: streamId(block), number: block.number};
  button.onclick = () => {
    if (!eligible(link)) return;
    visit = link;
    showTarget();
  };
  actions.append(button);
  top.append(identity, status, actions);
  const evidence = node("div", undefined, "evidence");
  const more = node("div", undefined, "more");
  more.append(components(block), history(block));
  evidence.append(sampleRow(block, block.trace.at(-1), true), more);
  panel.append(top, evidence);
  return panel;
}

function heading(title, path) {
  const box = node("header");
  const row = node("div", undefined, "title-row");
  row.append(node("h1", title));
  box.append(node("p", path, "crumb"), row);
  return box;
}

function showTarget() {
  if (!eligible(visit)) return;
  const block = blocks().find(item => item.number === visit.number);
  const stream = (current() ? run().formal : run().objects)[visit.stream];
  const pane = node("article", undefined, "target");
  const selected = node("h2", stream.id);
  selected.tabIndex = -1;
  const fields = facts([
    ...bindings(block),
    [current() ? tr("生产端返回总摩尔流量", "Returned total molar flow") : tr("工程输入总摩尔流量", "Project input molar flow"), stream.flow + " mol/s"],
    [tr("温度", "Temperature"), stream.temperature + " K"],
    [tr("压力", "Pressure"), stream.pressure + " Pa"],
    ...Object.entries(current() ? stream.components : stream.fractions).map(([id, value]) =>
      [id + " · " + (current() ? tr("组分摩尔流量", "Component molar flow") : tr("输入摩尔分数", "Input mole fraction")), current() ? value + " mol/s" : String(value)])
  ]);
  const back = node("button", "← " + tr("返回", "Return to ") + blockName(block));
  back.onclick = () => {
    if (!eligible(visit)) return;
    visit = null;
    destination.hidden = true;
    destination.replaceChildren();
    review.hidden = false;
    byId("locate-" + block.number).focus();
  };
  pane.append(node("p", runName() + " · " + blockName(block) + " · " +
    (current() ? tr("整图 Current 快照", "Current flowsheet snapshot") : tr("本次无正式流股结果", "No published stream results for this run")), "note"),
    selected, fields, back);
  destination.replaceChildren(heading(current() ? tr("正式流股", "Published stream") : tr("工程流股", "Project stream"),
    current() ? tr("结果 / 流股表", "Results / Stream table") : tr("变量浏览器 / 工程对象", "Variable browser / Project objects")), pane);
  review.hidden = true;
  destination.hidden = false;
  selected.focus();
}

function render() {
  review.hidden = false;
  destination.hidden = true;
  destination.replaceChildren();
  const isEn = language.value === "en";
  document.documentElement.lang = isEn ? "en" : "zh-CN";
  for (const [id, zh, en] of [
    ["review-label", "交互稿评审", "Prototype review"], ["case-label", "工况", "Case"], ["context-label", "来源", "Context"],
    ["language-label", "语言", "Language"], ["size-label", "字号", "Text size"], ["project-name", "Q7 串联回流", "Q7 serial recycle"]
  ]) byId(id).textContent = tr(zh, en);
  const options = [["原运行", "Original run"], ["修改输入", "Input edited"], ["重开同一工程", "Same project reopened"], ["新回执已接收", "New receipt accepted"]];
  options.forEach(([zh, en], i) => { context.options[i].textContent = tr(zh, en); });
  review.replaceChildren(heading(tr("流程结果", "Flowsheet results"), tr("流程运行 / 稳态运行计划", "Flow run / Steady-state plan")));
  if (context.value !== "original") {
    const box = node("section", undefined, "unavailable");
    const reasons = {
      edited: tr("输入已修改，原运行的来源已失效。", "Inputs changed. The original run no longer qualifies."),
      reopened: tr("已重开同一工程，原运行属于上一次会话。", "This project was reopened. The original run belongs to the previous session."),
      replaced: tr("已接收新回执，原运行的详情已关闭。", "A new receipt was accepted. The previous run detail has closed.")
    };
    box.append(node("h2", reasons[context.value]), node("p", runName() + tr(" 的流股定位已关闭。", " stream links are closed."), "note"));
    review.append(box);
    return;
  }
  const title = node("div", undefined, "ledger-title");
  title.append(node("h2", tr("回流块", "Recycle blocks")),
    node("span", tr("第 4 块 → s-bleed → 第 5 块", "Block 4 → s-bleed → Block 5"), "note"));
  review.append(receipt(), title, ...blocks().map(blockPanel));
  if (visit) showTarget();
}

function switchSource() {
  generation++;
  visit = null;
  render();
}
if (new URLSearchParams(location.search).get("case") === "099") scene.value = "099";
scene.addEventListener("change", () => { context.value = "original"; switchSource(); });
context.addEventListener("change", switchSource);
language.addEventListener("change", render);
byId("size").addEventListener("change", event => { document.documentElement.style.fontSize = event.target.value + "px"; });
render();

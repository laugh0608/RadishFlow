"use strict";

// Two applied-plan validation outcomes from the Q9 case. Editing the selector only changes the draft.
const cases = {
  "s-recycle-2": { source: "splitter-2.outlet_a", sink: "mixer-3.inlet_b", ready: true },
  "s-bleed": { source: "splitter-1.outlet_b", sink: "mixer-3.inlet_a", ready: false },
};
const saved = "s-recycle-2";
const words = {
  zh: {
    review: "Q9 交互稿", edit: "第二条草稿流股", language: "语言", size: "字号（px）", restart: "重置工况",
    title: "稳态运行计划", method: "分块回流（固定温压）", run: "运行", bindings: "撕裂绑定",
    draft: "草稿未应用", synced: "与已应用计划一致", draftNotice: "运行使用已应用计划。保存时仅写入正式配置。",
    apply: "应用计划", restore: "恢复已保存", discard: "放弃草稿", owner: "已应用计划",
    revision: "计划修订", ready: "拓扑与绑定已就绪", blocked: "拓扑与绑定未就绪",
    runtime: "运行时继续检查输入与物性。", outside: "撕裂流股位于循环块外。",
    stream: "流股", source: "源端口", sink: "汇端口", result: "流程结果",
    current: "10 股正式结果已发布", stale: "计划已改变，重新运行后可查看正式结果。",
    failed: "在装配阶段结束，本次无正式结果。"
  },
  en: {
    review: "Q9 review", edit: "Second draft stream", language: "Language", size: "Font size (px)", restart: "Reset case",
    title: "Steady-state run plan", method: "Partitioned recycle (fixed T/P)", run: "Run", bindings: "Tear bindings",
    draft: "Unapplied draft", synced: "Matches applied plan", draftNotice: "Run uses the applied plan. Save writes the applied configuration.",
    apply: "Apply plan", restore: "Restore saved", discard: "Discard draft", owner: "Applied plan",
    revision: "Plan revision", ready: "Topology and bindings ready", blocked: "Topology and bindings not ready",
    runtime: "Run also checks physical inputs and the property package.", outside: "The tear stream is outside a cyclic block.",
    stream: "Stream", source: "Source port", sink: "Sink port", result: "Flow result",
    current: "10 formal stream results published", stale: "The plan changed. Rerun to view formal results.",
    failed: "Stopped during assembly. No formal results for this run."
  },
};
const byId = id => document.getElementById(id);
function updateText(node, value) {
  if (node.textContent !== value) node.textContent = value;
}
const setText = (id, value) => updateText(byId(id), value);

let applied, draft, revision, runNumber, result;
function reset() {
  applied = saved;
  draft = "s-bleed";
  revision = 0;
  runNumber = 1;
  result = "Current";
  render();
}

function render() {
  const language = byId("language").value;
  const text = words[language];
  document.documentElement.lang = language === "zh" ? "zh-CN" : "en";
  document.documentElement.style.fontSize = byId("size").value + "px";
  for (const node of document.querySelectorAll("[data-label]")) {
    updateText(node, text[node.dataset.label]);
  }
  const pending = draft !== applied;
  byId("candidate").value = draft;
  setText("edit-state", pending ? text.draft : text.synced);
  setText("draft-stream", draft);
  setText("draft-ports", cases[draft].source + " → " + cases[draft].sink);
  byId("draft-notice").hidden = !pending;
  byId("apply").setAttribute("aria-disabled", String(!pending));
  byId("discard").setAttribute("aria-disabled", String(!pending));

  const check = cases[applied];
  byId("readiness").dataset.ready = String(check.ready);
  setText("revision", text.revision + " " + revision);
  setText("verdict", check.ready ? text.ready : text.blocked);
  setText("reason", check.ready ? text.runtime : text.outside);
  byId("targets").hidden = check.ready;
  setText("target-stream", check.ready ? "" : applied);
  setText("target-source", check.ready ? "" : check.source);
  setText("target-sink", check.ready ? "" : check.sink);

  const fields = [result];
  if (result !== "Stale") fields.push(text.run + " #" + runNumber);
  fields.push(text[result.toLowerCase()]);
  setText("receipt", fields.join(" · "));
}

function applyPlan(stream) {
  if (applied !== stream) {
    applied = stream;
    revision += 1;
    result = "Stale";
  }
  draft = applied;
  render();
}
function run() {
  runNumber += 1;
  result = cases[applied].ready ? "Current" : "Failed";
  render();
}

byId("candidate").addEventListener("change", event => {
  draft = event.target.value;
  render();
});
byId("apply").addEventListener("click", () => {
  if (draft !== applied) applyPlan(draft);
});
byId("discard").addEventListener("click", () => {
  if (draft !== applied) {
    draft = applied;
    render();
  }
});
byId("restore").addEventListener("click", () => applyPlan(saved));
byId("run").addEventListener("click", run);
byId("restart").addEventListener("click", reset);
byId("language").addEventListener("change", render);
byId("size").addEventListener("change", render);
document.addEventListener("keydown", event => {
  if (event.key === "F5" && !event.ctrlKey && !event.metaKey && !event.altKey) {
    event.preventDefault();
    if (!event.repeat) run();
  }
});
reset();

"use strict";

const element = id => document.getElementById(id);
const tr = (zh, en) => element("language").value === "zh" ? zh : en;
const caption = (id, zh, en) => { element(id).textContent = tr(zh, en); };
const unitNames = ["Feed A", "Feed B", "Mixer"];
const scenarios = [
  ["normal", "双进料 Mixer", "Two-feed mixer"],
  ["failure", "第二股缺乙烷", "Feed B missing ethane"],
  ["unsupported", "单相显热计划", "Single-phase sensible heat"],
];
let phase, completed, runNumber, currentResult, staleResult, feedB, pauseBeforeFirst;

function reset() {
  const kind = element("scenario").value || "normal";
  phase = kind === "unsupported" ? "unsupported" : "idle";
  completed = 0;
  runNumber = kind === "unsupported" ? 21 : 20;
  currentResult = kind === "normal" ? { run: 20, flow: 5 } : null;
  staleResult = null;
  feedB = 3;
  pauseBeforeFirst = false;
  render();
}

function start(firstStop) {
  runNumber += 1;
  completed = 0;
  pauseBeforeFirst = firstStop;
  phase = firstStop ? "pending" : "running";
  render();
}

// One click delivers one executor response. The review controls time independently of the product.
function advance() {
  if (phase === "cancelling") {
    phase = "cancelled";
  } else if (phase === "pending" && pauseBeforeFirst) {
    phase = "paused";
    pauseBeforeFirst = false;
  } else if (completed === 1 && element("scenario").value === "failure") {
    phase = "failed";
    currentResult = null;
  } else {
    completed += 1;
    if (completed === unitNames.length) {
      phase = "done";
      currentResult = { run: runNumber, flow: 2 + feedB };
      staleResult = null;
    } else if (phase === "pending" || phase === "stepping") {
      phase = "paused";
    }
  }
  render();
}

function statusLabel() {
  const labels = {
    idle: ["等待运行", "Ready to run"],
    running: ["运行中", "Running"],
    unsupported: ["运行中", "Running"],
    pending: ["正在暂停 · 等待单元边界", "Pause requested · waiting for a unit boundary"],
    paused: ["已暂停", "Paused"],
    stepping: ["单步执行中", "Stepping one unit"],
    cancelling: ["正在取消", "Cancellation requested"],
    cancelled: ["已取消", "Cancelled"],
    done: ["已完成", "Completed"],
    failed: ["求解失败", "Solve failed"],
  };
  return tr(...labels[phase]);
}

function position() {
  const unit = unitNames[completed];
  const progress = tr(`已完成 ${completed}/3`, `${completed}/3 completed`);
  switch (phase) {
    case "idle": return "";
    case "unsupported": return tr("此计划支持取消。", "This plan supports cancellation.");
    case "cancelled":
      return element("scenario").value === "unsupported" ? "" : progress;
    case "cancelling":
      return tr("等待线程退出，本次结果已禁止回写。", "Waiting for the worker to exit. Its result cannot be applied.");
    case "pending":
      return pauseBeforeFirst
        ? tr("正在准备本次运行。", "Preparing this run.")
        : tr(`${progress} · 等待 ${unit} 返回`, `${progress} · waiting for ${unit} to return`);
    case "paused": return tr(`${progress} · 下一单元 ${unit} 尚未开始`, `${progress} · next unit ${unit} has not started`);
    case "failed": return tr("Feed B 组成缺少乙烷。Mixer 未执行。", "Feed B is missing ethane. Mixer did not run.");
    case "done": return progress;
    default: return tr(`${progress} · 正在执行 ${unit}`, `${progress} · executing ${unit}`);
  }
}

function render() {
  const focused = document.activeElement;
  document.documentElement.lang = element("language").value === "zh" ? "zh-CN" : "en";
  document.documentElement.style.fontSize = element("size").value + "px";
  caption("review", "N3 交互评审", "N3 interaction review");
  caption("caseLabel", "案例", "Case");
  caption("languageLabel", "语言", "Language");
  caption("sizeLabel", "字号", "Text size");
  for (const [index, [value, zh, en]] of scenarios.entries()) {
    if (!element("scenario").options[index]) element("scenario").add(new Option("", value));
    element("scenario").options[index].text = tr(zh, en);
  }
  caption("advance", "演示下一次执行器回执", "Simulate next worker response");
  caption("edit", "模拟提交 Feed B 为 4 mol/s", "Simulate applying 4 mol/s to Feed B");
  caption("reset", "重置案例", "Reset case");
  caption("project", "双进料混合工程", "Two-feed mixer project");
  caption("runTitle", "流程运行", "Flowsheet run");
  caption("activityTitle", "运行状态", "Run status");
  caption("resultTitle", "已接收结果", "Accepted results");
  const unsupported = element("scenario").value === "unsupported";
  element("method").textContent = unsupported ? tr("单相显热", "Single-phase sensible heat") : tr("无回路稳态", "Acyclic steady state");
  element("inputs").textContent = `Feed A · 2 mol/s　 Feed B · ${feedB} mol/s`;
  caption("run", "运行", "Run");
  caption("startPaused", "运行并在首步前暂停", "Run and pause before first unit");
  caption("pause", "暂停", "Pause");
  caption("step", "单步（一个单元）", "Step one unit");
  caption("resume", "继续本次运行", "Continue this run");
  caption("cancel", "取消本次运行", "Cancel this run");
  const terminal = ["done", "failed", "cancelled"].includes(phase);
  const available = phase === "idle" || terminal;
  element("run").hidden = !available || unsupported;
  element("startPaused").hidden = !available || unsupported;
  element("pause").hidden = phase !== "running";
  element("step").hidden = element("resume").hidden = phase !== "paused";
  element("cancel").hidden = available || phase === "cancelling";
  element("advance").disabled = available || phase === "paused" || phase === "unsupported";
  element("edit").disabled = phase !== "paused" || element("scenario").value !== "normal" || feedB === 4;
  element("status").dataset.state = phase;
  element("status").textContent = (phase === "idle" ? "" : `#${runNumber} · `) + statusLabel();
  element("position").textContent = position();
  element("source").textContent = currentResult
    ? tr(`当前结果来自运行 #${currentResult.run}`, `Current result from run #${currentResult.run}`)
    : tr("暂无当前结果", "No current result");
  let outlet = "";
  if (currentResult) {
    outlet = tr(`Mixer 出口　${currentResult.flow} mol/s`, `Mixer outlet　${currentResult.flow} mol/s`);
  } else if (staleResult) {
    outlet = tr(`工程输入已修改，运行 #${staleResult.run} 的结果已过期。`, `Inputs changed. The result from run #${staleResult.run} is stale.`);
  }
  element("outlet").textContent = outlet;
  if (focused?.tagName === "BUTTON" && (focused.hidden || focused.disabled)) {
    // Only move focus when the previously focused control is no longer available.
    element("status").focus();
  }
}

element("scenario").onchange = reset;
element("language").onchange = element("size").onchange = render;
element("reset").onclick = reset;
element("run").onclick = () => start(false);
element("startPaused").onclick = () => start(true);
element("advance").onclick = advance;
element("edit").onclick = () => {
  feedB = 4;
  staleResult = currentResult;
  currentResult = null;
  phase = "cancelling";
  render();
};
for (const [id, next] of Object.entries({ pause: "pending", step: "stepping", resume: "running", cancel: "cancelling" })) {
  element(id).onclick = () => { phase = next; render(); };
}
reset();

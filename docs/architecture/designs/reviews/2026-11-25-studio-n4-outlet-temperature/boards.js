"use strict";

const el = id => document.getElementById(id);
const tr = (zh, en) => el("language").value === "zh" ? zh : en;
const units = ["Feed A", "Feed B", "Mixer", "Flash"];
const outlets = [
  { stream: "stream-feed-a", step: 1, port: "outlet" },
  { stream: "stream-feed-b", step: 2, port: "outlet" },
  { stream: "stream-mix-out", step: 3, port: "outlet" },
  { stream: "stream-liquid", step: 4, port: "liquid" },
  { stream: "stream-vapor", step: 4, port: "vapor" },
];
const captions = [...document.querySelectorAll("[data-en]")].map(node => [node, node.textContent]);
// Unrounded values read from the four D-05 CLI receipts.
const samples = {
  base: { feedB: 300, mixer: 300 },
  below: { feedB: 319.999999, mixer: 311.9999994 },
  equal: { feedB: 320, mixer: 312 },
  above: { feedB: 320.000001, mixer: 312.0000006 },
};
const bound = () => outlets.find(out => out.stream === el("outlet").value);
const busy = () => ["running", "pending", "paused", "stepping", "ending", "cancelling"].includes(phase);
let phase, completed, run, result, rule, observation, sample, inputUnit, canonical;
let invalidBinding, showValueError;

function outletName(out) {
  const port = { outlet: tr("物料出口", "outlet"), liquid: tr("液相出口", "liquid outlet"), vapor: tr("气相出口", "vapor outlet") };
  return units[out.step - 1] + " · " + port[out.port];
}

function readThreshold() {
  const raw = el("threshold").value.trim();
  if (!/^[+-]?(?:\d+(?:\.\d*)?|\.\d+)(?:e[+-]?\d+)?$/i.test(raw)) return null;
  const k = Number(raw) + (inputUnit === "°C" ? 273.15 : 0);
  return Number.isFinite(k) && k > 0 ? k : null;
}

function convertedText(k, unit) {
  if (unit === "K") return String(k);
  const c = k - 273.15;
  for (let digits = 1; digits <= 17; digits++) {
    const n = Number(c.toPrecision(digits));
    if (n + 273.15 === k) return String(n);
  }
  return String(c);
}

function reading(out) {
  if (out.step === 2) return sample.feedB;
  if (out.step === 3) return sample.mixer;
  return 300;
}

function reset() {
  const scenario = el("scenario").value;
  sample = samples[scenario] ?? samples.equal;
  run = 20;
  result = ["fresh", "failure", "rebind"].includes(scenario) ? null : { run, value: reading(outlets[2]) };
  phase = "idle";
  completed = 0;
  rule = null;
  observation = null;
  inputUnit = "°C";
  el("unit").value = inputUnit;
  el("threshold").value = "38.85";
  canonical = readThreshold();
  invalidBinding = scenario === "rebind";
  showValueError = false;
  el("outlet").value = invalidBinding ? "" : outlets[2].stream;
  render();
}

function start(watch) {
  if (busy()) return;
  if (watch && (!bound() || canonical === null)) {
    invalidBinding = !bound();
    showValueError = canonical === null;
    render();
    el(invalidBinding ? "outlet" : "threshold").focus();
    return;
  }
  run++;
  phase = "running";
  completed = 0;
  rule = watch ? { out: bound(), k: canonical, raw: el("threshold").value.trim(), unit: inputUnit } : null;
  if (rule) observation = { run, rule, state: "waiting" };
  render();
  el("status").focus();
}

function finish() {
  phase = "done";
  result = { run, value: reading(outlets[2]) };
}

function advance() {
  if (!busy() || phase === "paused") return;
  const scenario = el("scenario").value;
  if (phase === "pending") {
    phase = "paused";
  } else if (phase === "ending") {
    phase = "unavailable";
  } else if (phase === "cancelling") {
    phase = "cancelled";
    if (rule && observation.state === "waiting") observation.state = "unevaluated";
  } else {
    const waiting = rule && observation.state === "waiting";
    const next = phase === "stepping" ? completed + 1 : waiting ? rule.out.step : units.length;
    if (scenario === "failure" && completed < 2 && next >= 2) {
      completed = 1;
      phase = "failed";
      if (waiting) observation.state = "unevaluated";
    } else {
      completed = next;
      if (waiting && completed === rule.out.step) {
        if (scenario === "missing" && rule.out.step === 3) {
          observation.state = "missing";
          phase = "ending";
        } else {
          observation.value = reading(rule.out);
          observation.state = observation.value > rule.k ? "hit" : "miss";
          if (observation.state === "hit" && completed < units.length) phase = "pending";
        }
      }
      if (phase !== "ending" && completed === units.length) finish();
      else if (phase === "stepping") phase = "paused";
    }
  }
  render();
}

function render() {
  const focus = document.activeElement;
  document.documentElement.lang = tr("zh-CN", "en");
  document.documentElement.style.fontSize = el("size").value.split(" ")[0] + "px";
  for (const [node, zh] of captions) node.textContent = tr(zh, node.dataset.en);
  el("outlet").options[0].text = tr("重新选择物料出口", "Select the connected outlet");
  outlets.forEach((out, i) => { el("outlet").options[i + 1].text = outletName(out); });
  el("binding").textContent = bound()?.stream ?? "";
  el("conversion").textContent = canonical === null ? "" : tr("比较阈值", "Comparison threshold") + " · " + canonical + " K";
  el("outlet-error").textContent = invalidBinding ? tr("出口绑定已失效。请重新选择当前连接。", "The outlet connection changed. Select the current outlet.") : "";
  el("value-error").textContent = showValueError && canonical === null ? tr("请输入可换算为大于 0 K 的有限温度。", "Enter a finite absolute temperature above 0 K.") : "";
  el("outlet").setAttribute("aria-invalid", String(invalidBinding));
  el("threshold").setAttribute("aria-invalid", String(showValueError && canonical === null));
  el("ordinary").disabled = el("watch").disabled = busy();
  el("advance").disabled = !busy() || phase === "paused";
  el("step").hidden = el("resume").hidden = phase !== "paused";
  el("cancel").hidden = !["pending", "paused", "running", "stepping"].includes(phase);
  el("review-rule").hidden = phase !== "unavailable";
  el("run-id").textContent = phase === "idle" ? "" : "#" + run;
  el("activity").dataset.phase = phase;
  const states = {
    idle: ["就绪", "Ready"],
    running: ["运行中", "Running"],
    pending: ["条件已命中 · 等待单元边界", "Condition met · waiting for a unit boundary"],
    paused: ["已暂停", "Paused"],
    stepping: ["单步执行中", "Stepping one unit"],
    ending: ["监视值不可用 · 正在结束", "Observation unavailable · stopping"],
    unavailable: ["监视值不可用 · 本次运行已终止", "Observation unavailable · run stopped"],
    cancelling: ["正在取消", "Cancelling"],
    cancelled: ["已取消", "Cancelled"],
    done: ["已完成", "Completed"],
    failed: ["模型求解失败", "Model solve failed"],
  };
  el("status").textContent = tr(...states[phase]);
  let position = completed + "/4 " + tr("个单元已完成", "units completed");
  if (phase === "idle") position = "Feed A · 300 K · 2 mol/s　 /　 Feed B · " + sample.feedB + " K · 3 mol/s";
  if (phase === "paused") position += tr("，下一单元 ", " · next unit ") + units[completed] + tr(" 尚未开始。", " has not started.");
  if (phase === "failed") position += tr("。Feed B 组成缺少乙烷。", ". Feed B composition is missing ethane.");
  if (phase === "unavailable") position += tr("。Flash 未开始。本次没有发布结果。", ". Flash did not start. No result was published.");
  el("position").textContent = position;
  let cause = "";
  if (rule && observation.state === "hit" && completed === rule.out.step && ["pending", "paused"].includes(phase)) {
    cause = outletName(rule.out) + " · " + observation.value + " K > " + rule.k + " K";
  } else if (phase === "ending" || phase === "unavailable") {
    cause = outletName(rule.out) + " · " + rule.out.stream + tr(" 的温度缺失", " temperature missing");
  }
  el("trigger-note").textContent = cause;

  el("empty-record").textContent = observation ? "" : tr("尚无受监视运行的记录。", "No monitored run recorded yet.");
  el("record").hidden = !observation;
  el("evidence-id").textContent = observation ? "#" + observation.run : "";
  if (observation) {
    const { out, raw, unit, k } = observation.rule;
    const hasValue = observation.value !== undefined;
    const frozen = raw + " " + unit + (unit === "K" ? "" : " (" + k + " K)");
    el("observed-outlet").textContent = outletName(out) + " · " + out.stream;
    el("reading").textContent = hasValue ? observation.value + " K" : "";
    const messages = {
      waiting: tr("等待目标单元完成", "Waiting for the target unit"),
      unevaluated: tr("本次未求值", "Not evaluated in this run"),
      missing: tr("目标步骤完成时缺少出口温度", "Outlet temperature missing at step completion"),
      hit: out.step === units.length ? tr("命中于末步", "Condition met at the final step") : tr("高于阈值 · 已命中", "Above threshold · condition met"),
      miss: observation.value === k ? tr("等于阈值 · 未命中", "Equal to threshold · condition not met") : tr("低于阈值 · 未命中", "Below threshold · condition not met"),
    };
    el("comparison").textContent = messages[observation.state];
    el("source-step").textContent = tr("目标步 ", "Target step ") + out.step + "/4 · " + tr("本次阈值 ", "Run threshold ") + frozen;
  }
  el("result-source").textContent = result ? tr("当前结果来自运行 ", "Current result from run ") + "#" + result.run : tr("尚无当前结果", "No current result");
  el("result-value").textContent = result ? "Mixer · " + result.value + " K · 5 mol/s" : "";
  if (focus?.tagName === "BUTTON" && (focus.disabled || focus.hidden)) el("status").focus();
}

el("outlet").add(new Option("", ""));
el("outlet").options[0].disabled = true;
for (const out of outlets) el("outlet").add(new Option("", out.stream));
el("scenario").onchange = el("reset").onclick = reset;
el("language").onchange = el("size").onchange = render;
el("outlet").onchange = () => { invalidBinding = false; render(); };
el("threshold").oninput = () => { canonical = readThreshold(); render(); };
el("unit").onchange = () => {
  if (canonical === null) {
    el("unit").value = inputUnit;
    showValueError = true;
    el("threshold").focus();
  } else {
    inputUnit = el("unit").value;
    el("threshold").value = convertedText(canonical, inputUnit);
  }
  render();
};
el("rule-form").onsubmit = e => { e.preventDefault(); start(true); };
el("ordinary").onclick = () => start(false);
document.addEventListener("keydown", e => {
  if (e.key === "F5") { e.preventDefault(); if (!e.repeat) start(false); }
});
el("advance").onclick = advance;
el("review-rule").onclick = () => el("outlet").focus();
for (const [id, next] of [["step", "stepping"], ["resume", "running"], ["cancel", "cancelling"]]) {
  el(id).onclick = () => { phase = next; render(); };
}
reset();

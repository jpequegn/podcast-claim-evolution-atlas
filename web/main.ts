/// <reference types="vite/client" />
import init, {
  analyze,
  review,
  evaluate as evaluateWasm,
} from "./pkg/claim_wasm";
import {
  createIcons,
  Upload,
  Download,
  Search,
  Check,
  X,
  ExternalLink,
  GitCompareArrows,
} from "lucide";
import corpus from "../examples/corpus.json";
import "./style.css";

type Claim = (typeof corpus.claims)[number];
type Evidence = (typeof corpus.evidence)[number];
type Review = {
  request: {
    action: string;
    target: string;
    reviewer: string;
    at: string;
    note: string;
  };
  digest: string;
};
type Doc = {
  bundle: {
    title: string;
    claims: Claim[];
    evidence: Evidence[];
    aliases: Record<string, string>;
    version: number;
  };
  reviews: Review[];
};
type Edge = {
  proposal: {
    id: string;
    left: string;
    right: string;
    reason: string;
    evidence_ids: string[];
    relation: string;
  };
  relation: string;
  status: string;
  reviewer: string | null;
  closing_evidence: string[];
};
type Graph = {
  claims: Record<string, string>;
  edges: Edge[];
  revision: number;
  bundle_digest: string;
};
const app = document.querySelector<HTMLDivElement>("#app")!;
let doc: Doc = { bundle: corpus, reviews: [] };
let graph: Graph;
let selected = "",
  selectedEdge = "",
  tab = "timeline",
  error = "";
const filters = { search: "", topic: "", source: "", modality: "", status: "" };
const esc = (s: unknown) =>
  String(s).replace(
    /[&<>"']/g,
    (c) =>
      ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;" })[
        c
      ]!,
  );
const label = (s: string) => s.toLowerCase().replaceAll("_", " ");
const evidence = (c: Claim) =>
  doc.bundle.evidence.filter((e) => c.evidence_ids.includes(e.id));
const day = (c: Claim) =>
  evidence(c)
    .map((e) => e.asserted_at)
    .sort()[0] || "";
const visible = () =>
  doc.bundle.claims.filter(
    (c) =>
      (!filters.search ||
        [c.statement, c.subject, c.predicate]
          .join(" ")
          .toLowerCase()
          .includes(filters.search.toLowerCase())) &&
      (!filters.topic || c.topic === filters.topic) &&
      (!filters.modality || c.modality === filters.modality) &&
      (!filters.source ||
        evidence(c).some((e) => e.source === filters.source)) &&
      (!filters.status || graph.claims[c.id] === filters.status),
  );
const select = (id: string, title: string, values: string[], current: string) =>
  `<label>${title}<select id="${id}"><option value="">All ${title.toLowerCase()}</option>${[
    ...new Set(values),
  ]
    .sort()
    .map(
      (v) =>
        `<option ${current === v ? "selected" : ""} value="${esc(v)}">${esc(label(v))}</option>`,
    )
    .join("")}</select></label>`;
function evidencePanel(e: Evidence) {
  return `<section class="evidence"><span class="eyebrow">${esc(label(e.kind))}</span><h3>${esc(e.episode_title)}</h3><p>${esc(e.excerpt)}</p><div class="meta">${esc(e.source)} · ${esc(e.speaker)}<br>${e.asserted_at} · Segment ${esc(e.segment_id)} · ${e.kind === "transcript" ? Math.round((e.start_ms || 0) / 1000) + "s" : "No verified timestamp"}</div>${/^https?:/.test(e.uri) ? `<a href="${esc(e.uri)}" target="_blank" rel="noreferrer">Open source <i data-lucide="external-link"></i></a>` : ""}<details><summary>Provenance</summary><code>${esc(e.segment_digest)}</code><code>${esc(e.export_digest)}</code></details></section>`;
}
function reviewForm(
  target: string,
  actions: string[],
  ids: string[],
  relation?: string,
) {
  if (!actions.length) return "";
  return `<form id="review-form" data-target="${esc(target)}"><h3>Review decision</h3><label>Action<select name="action">${actions.map((a) => `<option value="${a}">${label(a)}</option>`).join("")}</select></label>${relation ? `<label>Relationship<select name="relation">${["REPEATS", "SUPPORTS", "CONTRADICTS", "REFINES", "NARROWS", "BROADENS", "TEMPORAL_CHANGE", "DEFINITION_MISMATCH"].map((r) => `<option ${r === relation ? "selected" : ""}>${r}</option>`).join("")}</select></label>` : ""}<label>Reviewer<input name="reviewer" required maxlength="80" value="operator"></label><label>Decision note<textarea name="note" required maxlength="1000"></textarea></label><label>Review date<input type="date" name="at" required value="${new Date().toISOString().slice(0, 10)}"></label><fieldset><legend>Evidence</legend><div class="evidence-checks">${doc.bundle.evidence
    .filter((e) => actions.includes("resolve") || ids.includes(e.id))
    .map(
      (e) =>
        `<label><input type="checkbox" name="evidence" value="${esc(e.id)}" ${ids.includes(e.id) ? "checked" : ""}>${esc(e.id)}</label>`,
    )
    .join(
      "",
    )}</div></fieldset><button class="primary" type="submit"><i data-lucide="check"></i>Record decision</button></form>`;
}
function detail() {
  const edge = graph.edges.find((e) => e.proposal.id === selectedEdge);
  if (edge) {
    const pair = [edge.proposal.left, edge.proposal.right].map((id) =>
      doc.bundle.claims.find((c) => c.id === id)!,
    );
    const actions =
      edge.status === "candidate"
        ? ["accept_relation", "reject_relation"]
        : edge.status === "accepted" && edge.relation === "CONTRADICTS"
          ? ["resolve", "supersede"]
          : [];
    return `<span class="eyebrow">Relationship / ${esc(edge.status)}</span><h2>${esc(label(edge.relation))}</h2><p class="reason">${esc(label(edge.proposal.reason))}</p>${pair.map((c) => `<button class="claim-link" data-claim="${esc(c.id)}">${esc(c.statement)} <span>${esc(graph.claims[c.id])}</span></button>`).join("")}${reviewForm(edge.proposal.id, actions, edge.proposal.evidence_ids, edge.relation)}${doc.bundle.evidence
      .filter((e) => edge.proposal.evidence_ids.includes(e.id))
      .map(evidencePanel)
      .join("")}`;
  }
  const c = doc.bundle.claims.find((c) => c.id === selected);
  if (!c) return "<h2>No claim selected</h2>";
  return `<span class="eyebrow">${esc(c.topic)} / ${esc(graph.claims[c.id])}</span><h2>${esc(c.statement)}</h2><dl><dt>Subject</dt><dd>${esc(c.subject)}</dd><dt>Modality</dt><dd>${esc(c.modality)} / ${esc(c.stance)}</dd><dt>Valid time</dt><dd>${c.scope.valid_from} to ${c.scope.valid_to}</dd><dt>Population</dt><dd>${esc(c.scope.population)}</dd><dt>System</dt><dd>${esc(c.scope.system)}</dd><dt>Definition</dt><dd>${esc(c.scope.definition)}</dd><dt>Conditions</dt><dd>${esc(c.scope.conditions.join(", ") || "None stated")}</dd><dt>Extractor</dt><dd>${esc(c.extractor)}</dd></dl>${evidence(c).map(evidencePanel).join("")}${reviewForm(c.id, graph.claims[c.id] === "candidate" ? ["approve_claim", "reject_claim"] : [], c.evidence_ids)}`;
}
function render() {
  const mainScroll = document.querySelector("main")?.scrollTop || 0;
  const rows = visible().sort(
    (a, b) => day(a).localeCompare(day(b)) || a.id.localeCompare(b.id),
  );
  const ids = new Set(rows.map((c) => c.id));
  const edges = graph.edges.filter(
    (e) => ids.has(e.proposal.left) && ids.has(e.proposal.right),
  );
  if (!ids.has(selected)) selected = rows[0]?.id || "";
  if (!edges.some((e) => e.proposal.id === selectedEdge)) selectedEdge = "";
  const synthetic = doc.bundle.evidence.every((e) => e.kind === "synthetic");
  app.innerHTML = `<header><div><span class="eyebrow">CASTFLOW / RESEARCH</span><h1>Claim Evolution Atlas</h1></div><div class="header-actions"><span class="local">${synthetic ? "Synthetic corpus" : "Local evidence"}</span><button id="import" class="icon" title="Open local atlas document" aria-label="Open local atlas document"><i data-lucide="upload"></i></button><button id="export" class="icon" title="Download atlas document" aria-label="Download atlas document"><i data-lucide="download"></i></button><input type="file" id="file" accept=".json" hidden></div></header><section class="summary"><div><h2>${esc(doc.bundle.title)}</h2><span>${doc.bundle.claims.length} claims · ${new Set(doc.bundle.evidence.map((e) => e.episode_id)).size} episodes · ${new Set(doc.bundle.evidence.map((e) => e.source)).size} source labels</span></div><div class="totals"><strong>${Object.values(graph.claims).filter((s) => s === "approved").length}<small>Approved claims</small></strong><strong>${graph.edges.filter((e) => e.relation === "CONTRADICTS" && e.status === "accepted").length}<small>Reviewed disputes</small></strong><strong>${graph.revision}<small>Review decisions</small></strong></div></section><div class="toolbar"><label class="search"><span>Search</span><input id="search" type="search" value="${esc(filters.search)}" placeholder="Find a claim"></label>${select(
    "topic",
    "Topics",
    doc.bundle.claims.map((c) => c.topic),
    filters.topic,
  )}${select(
    "source",
    "Sources",
    doc.bundle.evidence.map((e) => e.source),
    filters.source,
  )}${select(
    "modality",
    "Modalities",
    doc.bundle.claims.map((c) => c.modality),
    filters.modality,
  )}${select("status", "Review states", ["candidate", "approved", "rejected"], filters.status)}</div><div id="error" role="alert" ${error ? "" : "hidden"}>${esc(error)}</div><div class="workspace"><main><nav aria-label="Views">${["timeline", "relationships", "history"].map((t) => `<button data-tab="${t}" aria-pressed="${tab === t}">${t === "timeline" ? "Timeline" : t === "relationships" ? "Relationships" : "Review history"}</button>`).join("")}<span>${rows.length} visible claims</span></nav>${tab === "timeline" ? `<div class="chart"><canvas id="chart" aria-label="Claim assertion dates by topic" role="img"></canvas></div><div class="claim-table"><div class="table-head"><span>Assertion date / source</span><span>Claim</span><span>Review</span></div>${rows.map((c) => `<button class="claim-row ${selected === c.id && !selectedEdge ? "selected" : ""}" data-claim="${esc(c.id)}"><span><b>${day(c)}</b><small>${esc(evidence(c)[0]?.source)}</small></span><span>${esc(c.statement)}<small>${esc(c.predicate)} · ${esc(c.modality)}</small></span><span class="badge ${graph.claims[c.id]}">${graph.claims[c.id]}</span></button>`).join("") || '<p class="empty">No matching claims</p>'}</div>` : tab === "relationships" ? `<div class="matrix">${edges.map((e) => `<button class="edge-row ${selectedEdge === e.proposal.id ? "selected" : ""}" data-edge="${e.proposal.id}"><span class="relation ${e.relation === "CONTRADICTS" ? "conflict" : ""}">${esc(label(e.relation))}</span><span>${esc(e.proposal.left)}<br>${esc(e.proposal.right)}</span><span class="badge">${esc(e.status)}</span></button>`).join("") || '<p class="empty">No pairs match the selected claims</p>'}</div>` : `<ol class="history">${doc.reviews.map((r) => `<li><b>${esc(label(r.request.action))}</b><span>${esc(r.request.reviewer)} · ${r.request.at}</span><p>${esc(r.request.note)}</p><code>${esc(r.request.target)}</code></li>`).join("") || "<li>No review decisions recorded</li>"}</ol>`}</main><aside id="detail">${detail()}</aside></div><footer>LOCAL WASM · revision ${graph.revision}<span>${esc(graph.bundle_digest.slice(0, 16))}</span></footer>`;
  createIcons({
    icons: {
      Upload,
      Download,
      Search,
      Check,
      X,
      ExternalLink,
      GitCompareArrows,
    },
  });
  if (tab === "timeline") draw(rows);
  document.querySelector("main")!.scrollTop = mainScroll;
  document
    .querySelector("#import")!
    .addEventListener("click", () =>
      document.querySelector<HTMLInputElement>("#file")!.click(),
    );
  document.querySelector("#export")!.addEventListener("click", () => {
    const a = document.createElement("a");
    const url = URL.createObjectURL(
      new Blob([JSON.stringify(doc, null, 2)], { type: "application/json" }),
    );
    a.href = url;
    a.download = "claim-atlas.json";
    a.click();
    setTimeout(() => URL.revokeObjectURL(url), 1000);
  });
  document.querySelector("#file")!.addEventListener("change", async (e) => {
    try {
      const f = (e.target as HTMLInputElement).files?.[0];
      if (!f) return;
      if (f.size > 4_000_000) throw Error("Document exceeds 4 MB");
      const raw = await f.text();
      const next = JSON.parse(analyze(raw));
      doc = JSON.parse(raw);
      graph = next;
      selected = "";
      selectedEdge = "";
      Object.assign(filters, {
        search: "",
        topic: "",
        source: "",
        modality: "",
        status: "",
      });
      error = "";
      render();
    } catch (e) {
      error = String(e);
      render();
    }
  });
  for (const key of Object.keys(filters) as (keyof typeof filters)[]) {
    document
      .getElementById(key)!
      .addEventListener(key === "search" ? "input" : "change", (e) => {
        const el = e.target as HTMLInputElement;
        const pos = el.selectionStart;
        filters[key] = el.value;
        render();
        if (key === "search") {
          const input = document.querySelector<HTMLInputElement>("#search")!;
          input.focus();
          if (pos !== null) input.setSelectionRange(pos, pos);
        }
      });
  }
  document.querySelectorAll<HTMLElement>("[data-claim]").forEach((el) =>
    el.addEventListener("click", () => {
      selected = el.dataset.claim!;
      selectedEdge = "";
      render();
      if (innerWidth <= 760)
        document.querySelector("aside")!.scrollIntoView({ behavior: "smooth" });
    }),
  );
  document.querySelectorAll<HTMLElement>("[data-edge]").forEach((el) =>
    el.addEventListener("click", () => {
      selectedEdge = el.dataset.edge!;
      render();
      if (innerWidth <= 760)
        document.querySelector("aside")!.scrollIntoView({ behavior: "smooth" });
    }),
  );
  document.querySelectorAll<HTMLElement>("[data-tab]").forEach((el) =>
    el.addEventListener("click", () => {
      tab = el.dataset.tab!;
      render();
    }),
  );
  document
    .querySelector<HTMLFormElement>("#review-form")
    ?.addEventListener("submit", (e) => {
      e.preventDefault();
      const form = e.target as HTMLFormElement;
      const fd = new FormData(form);
      const action = String(fd.get("action"));
      try {
        const req = {
          revision: graph.revision,
          action,
          target: form.dataset.target,
          relation: action === "accept_relation" ? fd.get("relation") : null,
          reviewer: fd.get("reviewer"),
          note: fd.get("note"),
          evidence_ids: fd.getAll("evidence"),
          at: fd.get("at"),
          elapsed_seconds: null,
        };
        doc = JSON.parse(review(JSON.stringify(doc), JSON.stringify(req)));
        graph = JSON.parse(analyze(JSON.stringify(doc)));
        error = "";
        render();
      } catch (e) {
        error = String(e);
        const box = document.querySelector<HTMLDivElement>("#error")!;
        box.hidden = false;
        box.textContent = error;
      }
    });
}
function draw(rows: Claim[]) {
  const canvas = document.querySelector<HTMLCanvasElement>("#chart")!;
  const w = canvas.parentElement!.clientWidth,
    h = 170,
    ratio = devicePixelRatio || 1;
  canvas.width = w * ratio;
  canvas.height = h * ratio;
  canvas.style.width = w + "px";
  canvas.style.height = h + "px";
  const ctx = canvas.getContext("2d")!;
  ctx.scale(ratio, ratio);
  const topics = [...new Set(rows.map((c) => c.topic))];
  const dates = rows.map((c) => Date.parse(day(c)));
  const min = Math.min(...dates),
    max = Math.max(...dates);
  const colors = ["#087f78", "#b33e65", "#66634e"];
  const points: { x: number; y: number; id: string }[] = [];
  ctx.fillStyle = "#f7faf9";
  ctx.fillRect(0, 0, w, h);
  ctx.font = "11px system-ui";
  topics.forEach((t, i) => {
    const y = 32 + i * 38;
    ctx.strokeStyle = "#dfe8e5";
    ctx.beginPath();
    ctx.moveTo(125, y);
    ctx.lineTo(w - 18, y);
    ctx.stroke();
    ctx.fillStyle = "#53625e";
    ctx.fillText(t, 12, y + 4);
  });
  rows.forEach((c, i) => {
    const x = 135 + ((w - 160) * (dates[i] - min)) / (max - min || 1);
    const j = topics.indexOf(c.topic);
    const y = 32 + j * 38;
    ctx.beginPath();
    ctx.arc(x, y, c.id === selected ? 6 : 4, 0, Math.PI * 2);
    ctx.fillStyle = colors[j % colors.length];
    ctx.fill();
    points.push({ x, y, id: c.id });
  });
  if (rows.length) {
    ctx.fillStyle = "#53625e";
    ctx.fillText(new Date(min).toISOString().slice(0, 10), 125, 155);
    ctx.fillText(
      new Date(max).toISOString().slice(0, 10),
      Math.max(125, w - 90),
      155,
    );
  }
  canvas.onclick = (e) => {
    const box = canvas.getBoundingClientRect();
    const hit = points.find(
      (p) =>
        Math.hypot(p.x - (e.clientX - box.left), p.y - (e.clientY - box.top)) <
        9,
    );
    if (hit) {
      selected = hit.id;
      selectedEdge = "";
      render();
    }
  };
}
await init();
graph = JSON.parse(analyze(JSON.stringify(doc)));
render();
window.addEventListener("resize", () => {
  if (tab === "timeline") draw(visible());
});
// Same serialized entry points are used by native/WASM parity checks.
export { analyze, review, evaluateWasm };

import { useEffect, useMemo, useState } from "react";
import { PageTitle } from "../components/PageTitle";
import { MnemoSection } from "../components/MnemoSection";
import { ContentContainer } from "../components/ContentContainer";
import type { JobDTO } from "../types/jobs";
import { fetchJobs } from "../api/jobs";
import { MnemoSkeleton } from "../components/MnemoSkeleton";
import { mnemoPost } from "../api/client";
import { JobStatusBadge } from "../components/JobStatusBadge";
import { JobDetailsModal } from "../components/JobDetailsModal";
import { JobProgressBar } from "../components/JobProgressBar";
import { JobsTimeline } from "../components/JobsTimeline";
import { useWS } from "../context/WSContext";
import { JobTimeline } from "../components/JobTimeline";
import { mnemoGet } from "../api/client";
import { JobTypeBadge } from "../components/JobTypeBadge";
import { formatDate } from "../utils/date";
import { useAppState } from "../context";

type Metrics = {
  documents: number;
  chunks: number;
  embeddings: number;
  qdrant_writes: number;
  duration_ms: number;
};

const formatDuration = (ms: number) => {
  if (!ms || ms <= 0) return "00:00:00";
  const totalSeconds = Math.floor(ms / 1000);
  const h = Math.floor(totalSeconds / 3600);
  const m = Math.floor((totalSeconds % 3600) / 60);
  const s = totalSeconds % 60;
  const pad = (n: number) => n.toString().padStart(2, "0");
  return `${pad(h)}:${pad(m)}:${pad(s)}`;
};

export function PipelineMonitor() {
  const columns = ["job", "status", "created", "timeline"];
  const [rows, setRows] = useState<JobDTO[]>([]);
  const [selected, setSelected] = useState<JobDTO | null>(null);
  const [loading, setLoading] = useState(true);
  const [runningIds, setRunningIds] = useState<Set<string>>(new Set());
  const { messages, heartbeat, ingestionSteps } = useWS();
  const [metrics, setMetrics] = useState<Metrics | null>(null);
  const [showHistory, setShowHistory] = useState(true);
  const [recentJobs, setRecentJobs] = useState<JobDTO[]>([]);
  const [errorSummary, setErrorSummary] = useState<string[]>([]);
  const { addToast } = useAppState() || {};

  const deriveStatus = (job: JobDTO): JobDTO["status"] => {
    const steps = ingestionSteps[job.id] || {};
    const values = Object.values(steps).map((s) => s?.toLowerCase?.() || s);
    if (values.includes("failed") || values.includes("panic")) return "failed";
    if (steps["completed"] === "done") return "success";
    if (values.includes("running")) return "running";
    return job.status;
  };

  const historyJobs = useMemo(() => {
    const deriveStatus = (job: JobDTO): JobDTO["status"] => {
      const steps = ingestionSteps[job.id] || {};
      const values = Object.values(steps).map((s) => s?.toLowerCase?.() || s);
      if (values.includes("failed") || values.includes("panic"))
        return "failed";
      if (steps["completed"] === "done") return "success";
      if (values.includes("running")) return "running";
      return job.status;
    };
    return rows
      .map((r) => ({ ...r, status: deriveStatus(r) }))
      .filter((r) => r.status === "success" || r.status === "failed")
      .sort(
        (a, b) =>
          new Date(b.created_at).getTime() - new Date(a.created_at).getTime(),
      )
      .slice(0, 10);
  }, [rows, ingestionSteps]);

  useEffect(() => {
    // Whenever history changes, load metrics for the most recent successful job.
    const lastSuccess = [...historyJobs].sort(
      (a, b) =>
        new Date(b.created_at).getTime() - new Date(a.created_at).getTime(),
    )[0];
    if (lastSuccess) {
      loadMetrics(lastSuccess.id);
    }
  }, [historyJobs]);

  async function loadJobs() {
    setLoading(true);
    const data = await fetchJobs(true);
    setRows(data);
    setRecentJobs(
      [...data]
        .sort(
          (a, b) =>
            new Date(b.updated_at).getTime() - new Date(a.updated_at).getTime(),
        )
        .slice(0, 5),
    );
    setLoading(false);
  }

  async function startJob() {
    try {
      await mnemoPost("/v1/jobs/create", { job_type: "filesystem_scan" });
      await loadJobs();
    } catch (e) {
      console.error("Failed to start job", e);
    }
  }

  async function loadMetrics(jobId?: string) {
    try {
      const res = await mnemoGet(
        jobId
          ? `/v1/ingestion/metrics?job_id=${jobId}`
          : "/v1/ingestion/metrics",
      );
      setMetrics(res);
    } catch (e) {
      // ignore for now
    }
  }

  async function runJob(jobId: string) {
    // Disable all Run buttons while a job is running.
    setRunningIds((prev) => {
      const next = new Set(prev);
      next.add(jobId);
      return next;
    });
    try {
      await mnemoPost("/v1/jobs/run", { job_id: jobId });
      await loadJobs();
    } catch (e) {
      console.error("Failed to run job", e);
      alert("Ingestion failed");
    } finally {
      setRunningIds((prev) => {
        const next = new Set(prev);
        next.delete(jobId);
        return next;
      });
    }
  }

  async function abortJob(jobId: string) {
    setRunningIds((prev) => {
      const next = new Set(prev);
      next.delete(jobId);
      return next;
    });
    try {
      await mnemoPost("/v1/jobs/abort", { job_id: jobId });
      await loadJobs();
    } catch (e) {
      console.error("Failed to abort job", e);
    }
  }

  useEffect(() => {
    loadJobs();
    loadMetrics();
    const interval = setInterval(() => {
      loadMetrics();
    }, 5000);
    return () => clearInterval(interval);
  }, []);

  // Watchdog: if any job is running but we haven't seen WS traffic for 10s, mark it failed locally.
  useEffect(() => {
    const timeout = setTimeout(() => {
      setRows((prev) =>
        prev.map((job) => {
          if (job.status === "running") {
            return { ...job, status: "failed", progress: 0 };
          }
          return job;
        }),
      );
      setRecentJobs((prev) =>
        prev.map((job) => {
          if (job.status === "running") {
            return { ...job, status: "failed", progress: 0 };
          }
          return job;
        }),
      );
    }, 10_000);
    return () => clearTimeout(timeout);
  }, [heartbeat]);

  const latestJobEvent = useMemo(() => {
    const jobEvents = messages.filter(
      (m) => m?.event === "job_update" || m?.event === "jobs_snapshot",
    );
    return jobEvents[jobEvents.length - 1];
  }, [messages]);

  useEffect(() => {
    const latestErrorSummary = messages.findLast(
      (m) => m?.event === "ingest_error_summary",
    );
    if (latestErrorSummary && Array.isArray(latestErrorSummary.errors)) {
      setErrorSummary(latestErrorSummary.errors as string[]);
      addToast?.("Pipeline crashed. Check logs.");
      // Mark any in-flight jobs as failed to reflect pipeline crash.
      setRows((prev) =>
        prev.map((job) => {
          if (job.status === "running" || job.status === "pending") {
            return { ...job, status: "failed", progress: 0 };
          }
          return job;
        }),
      );
      setRecentJobs((prev) =>
        prev.map((job) => {
          if (job.status === "running" || job.status === "pending") {
            return { ...job, status: "failed", progress: 0 };
          }
          return job;
        }),
      );
    }
  }, [messages]);

  useEffect(() => {
    if (!latestJobEvent) return;

    // Snapshot contains full list
    if (
      latestJobEvent.event === "jobs_snapshot" &&
      Array.isArray(latestJobEvent.jobs)
    ) {
      setRows(latestJobEvent.jobs);
      setRecentJobs(
        [...latestJobEvent.jobs]
          .sort(
            (a, b) =>
              new Date(b.updated_at).getTime() -
              new Date(a.updated_at).getTime(),
          )
          .slice(0, 5),
      );
      setLoading(false);
      return;
    }

    // Single job update
    if (latestJobEvent.event === "job_update" && latestJobEvent.job_id) {
      setRows((prev) => {
        let found = false;
        const next = prev.map((job) => {
          if (job.id === latestJobEvent.job_id) {
            found = true;
            return {
              ...job,
              status: (latestJobEvent.status as JobDTO["status"]) ?? job.status,
              progress:
                typeof latestJobEvent.progress === "number"
                  ? latestJobEvent.progress
                  : job.progress,
              updated_at: latestJobEvent.updated_at ?? job.updated_at,
            };
          }
          return job;
        });

        if (!found) {
          next.unshift({
            id: latestJobEvent.job_id,
            job_type: latestJobEvent.job_type ?? "filesystem_scan",
            status: (latestJobEvent.status as JobDTO["status"]) ?? "pending",
            created_at: latestJobEvent.created_at ?? new Date().toISOString(),
            updated_at: latestJobEvent.updated_at ?? new Date().toISOString(),
            progress:
              typeof latestJobEvent.progress === "number"
                ? latestJobEvent.progress
                : 0,
          });
        }

        return next;
      });
      setRecentJobs((prev) => {
        const updated = (() => {
          let found = false;
          const mapped = prev.map((job) => {
            if (job.id === latestJobEvent.job_id) {
              found = true;
              return {
                ...job,
                status:
                  (latestJobEvent.status as JobDTO["status"]) ?? job.status,
                progress:
                  typeof latestJobEvent.progress === "number"
                    ? latestJobEvent.progress
                    : job.progress,
                updated_at: latestJobEvent.updated_at ?? job.updated_at,
              };
            }
            return job;
          });
          if (!found) {
            mapped.unshift({
              id: latestJobEvent.job_id,
              job_type: latestJobEvent.job_type ?? "filesystem_scan",
              status: (latestJobEvent.status as JobDTO["status"]) ?? "pending",
              created_at: latestJobEvent.created_at ?? new Date().toISOString(),
              updated_at: latestJobEvent.updated_at ?? new Date().toISOString(),
              progress:
                typeof latestJobEvent.progress === "number"
                  ? latestJobEvent.progress
                  : 0,
            });
          }
          return mapped;
        })();

        return updated
          .sort(
            (a, b) =>
              new Date(b.updated_at).getTime() -
              new Date(a.updated_at).getTime(),
          )
          .slice(0, 5);
      });
      setLoading(false);
      if (latestJobEvent.status === "success") {
        loadMetrics(latestJobEvent.job_id);
      }
    }
  }, [latestJobEvent]);

  return (
    <ContentContainer>
      <div className="max-w-[1600px] w-full mx-auto">
        <section className="space-y-6 mb-8">
          <PageTitle
            title="Pipeline Monitor"
            subtitle="Monitor ingestion pipelines"
          />
          {errorSummary.length > 0 && (
            <div className="rounded-xl border border-red-700/50 bg-red-900/30 p-4 text-red-100">
              <div className="font-semibold mb-2">Ingestion errors</div>
              <ul className="list-disc list-inside space-y-1 text-sm">
                {errorSummary.map((e, idx) => (
                  <li key={`${e}-${idx}`}>{e}</li>
                ))}
              </ul>
            </div>
          )}
          {metrics && (
            <div className="rounded-xl border border-[rgba(212,166,87,0.2)] bg-[var(--mnemo-bg-2)] p-4 mnemo-shadow-1">
              <div className="text-sm font-semibold mb-2 text-[var(--mnemo-text)]">
                Last Ingestion Summary
              </div>
              <div className="grid gap-3 grid-cols-2 md:grid-cols-4 text-sm">
                <div className="rounded border border-[rgba(212,166,87,0.15)] bg-[var(--mnemo-bg-2)] p-3">
                  <div className="text-xs text-gray-400">Documents</div>
                  <div className="text-lg font-semibold">
                    {metrics.documents}
                  </div>
                </div>
                <div className="rounded border border-[rgba(212,166,87,0.15)] bg-[var(--mnemo-bg-2)] p-3">
                  <div className="text-xs text-gray-400">Chunks</div>
                  <div className="text-lg font-semibold">{metrics.chunks}</div>
                </div>
                <div className="rounded border border-[rgba(212,166,87,0.15)] bg-[var(--mnemo-bg-2)] p-3">
                  <div className="text-xs text-gray-400">Vectors</div>
                  <div className="text-lg font-semibold">
                    {metrics.qdrant_writes}
                  </div>
                </div>
                <div className="rounded border border-[rgba(212,166,87,0.15)] bg-[var(--mnemo-bg-2)] p-3">
                  <div className="text-xs text-gray-400">Duration</div>
                  <div className="text-lg font-semibold">
                    {metrics.duration_ms
                      ? formatDuration(metrics.duration_ms)
                      : "—"}
                  </div>
                </div>
              </div>
            </div>
          )}
          <MnemoSection title="Ingestion Jobs">
            <div className="mb-2 flex justify-end">
              <button
                className="rounded px-4 py-2 text-sm border border-[rgba(212,166,87,0.3)] bg-[var(--mnemo-bg-2)] text-[var(--mnemo-text)] mnemo-shadow-1 hover:mnemo-glow-gold"
                onClick={startJob}
              >
                Start Filesystem Scan
              </button>
            </div>
            <div className="overflow-hidden rounded-xl max-h-[450px] overflow-y-auto">
              <table className="w-full text-[var(--mnemo-text)] border-separate border-spacing-y-2 border-spacing-x-0">
                <thead>
                  <tr className="bg-black/30">
                    {columns.map((c) => (
                      <th key={c} className="px-3 py-2 text-left">
                        {c}
                      </th>
                    ))}
                  </tr>
                </thead>
                <tbody>
                  {loading ? (
                    <tr>
                      <td colSpan={columns.length} className="px-3 py-4">
                        <MnemoSkeleton />
                      </td>
                    </tr>
                  ) : !Array.isArray(recentJobs) || recentJobs.length === 0 ? (
                    <tr>
                      <td
                        colSpan={columns.length}
                        className="px-3 py-4 text-sm opacity-70"
                      >
                        No jobs executed yet.
                      </td>
                    </tr>
                  ) : (
                    recentJobs.map((job) => {
                      const displayStatus = deriveStatus(job);
                      return (
                        <tr
                          key={job.id}
                          className={`border-t cursor-pointer hover:bg-[var(--mnemo-bg-2)] transition-all ${
                            selected?.id === job.id
                              ? "bg-[var(--mnemo-bg-3)] border-[var(--mnemo-accent)]"
                              : displayStatus === "pending"
                                ? "mnemo-glow-gold border-black/20 bg-yellow-500/20"
                                : displayStatus === "running"
                                  ? "animate-pulse border-blue-500/50 bg-blue-500/20"
                                  : displayStatus === "success"
                                    ? "border-green-500/40 mnemo-shadow-2 bg-green-500/20"
                                    : displayStatus === "failed" &&
                                        (messages.some(
                                          (m) =>
                                            (m.event === "ingest_step" &&
                                              m.step === "panic" &&
                                              m.job_id === job.id) ||
                                            (m.event === "pipeline_failed" &&
                                              m.job_id === job.id),
                                        )
                                          ? true
                                          : false)
                                      ? "border-red-500 bg-red-700/20 animate-pulse"
                                      : "border-red-500/50 bg-red-500/20"
                          }`}
                          onClick={() => setSelected(job)}
                          style={{ animation: "fadeIn 0.3s ease" }}
                        >
                          {columns.map((c) => (
                            <td
                              key={c}
                              className="px-3 py-2 opacity-80 align-top"
                            >
                              {c === "job" && (
                                <div className="space-y-1">
                                  <div className="flex items-center gap-2">
                                    <span>{job.job_type}</span>
                                    <JobTypeBadge kind={job.job_type} />
                                  </div>
                                  {displayStatus === "running" && (
                                    <JobProgressBar
                                      percent={Number(
                                        (job as any).progress ?? 40,
                                      )}
                                    />
                                  )}
                                  {(displayStatus === "pending" ||
                                    displayStatus === "running") && (
                                    <div className="transition-all duration-200 space-y-1">
                                      <button
                                        className="mt-1 rounded px-2 py-1 text-xs border border-[rgba(212,166,87,0.3)] bg-[var(--mnemo-bg-2)] text-[var(--mnemo-text)] hover:mnemo-glow-gold"
                                        onClick={(e) => {
                                          e.stopPropagation();
                                          runJob(job.id);
                                        }}
                                        disabled={runningIds.size > 0}
                                      >
                                        {runningIds.has(job.id)
                                          ? "Running…"
                                          : "Run now"}
                                      </button>
                                      {displayStatus === "running" && (
                                        <button
                                          className="mt-1 rounded px-2 py-1 text-xs border border-red-500/40 bg-red-500/10 text-red-200 hover:bg-red-500/20"
                                          onClick={(e) => {
                                            e.stopPropagation();
                                            abortJob(job.id);
                                          }}
                                        >
                                          Abort
                                        </button>
                                      )}
                                    </div>
                                  )}
                                </div>
                              )}
                              {c === "status" && (
                                <JobStatusBadge status={displayStatus} />
                              )}
                              {c === "created" && formatDate(job.created_at)}
                              {c === "timeline" && (
                                <JobTimeline status={displayStatus} />
                              )}
                            </td>
                          ))}
                        </tr>
                      );
                    })
                  )}
                </tbody>
              </table>
            </div>
          </MnemoSection>
          <MnemoSection title="History">
            <div className="mb-2">
              <button
                className="rounded px-3 py-1 text-xs border border-[rgba(212,166,87,0.3)] bg-[var(--mnemo-bg-2)] text-[var(--mnemo-text)] hover:mnemo-glow-gold"
                onClick={() => setShowHistory((v) => !v)}
              >
                {showHistory ? "Hide history" : "Show history"}
              </button>
            </div>
            {showHistory && (
              <div className="grid gap-4 md:grid-cols-2 grid-cols-1">
                {loading ? (
                  <MnemoSkeleton />
                ) : (
                  historyJobs.slice(0, 50).map((job) => (
                    <div
                      key={job.id}
                      className="rounded-lg border border-[rgba(212,166,87,0.15)] bg-[var(--mnemo-bg-2)] p-3 mnemo-shadow-1 flex items-center justify-between"
                    >
                      <div className="flex items-center gap-2">
                        <div className="text-sm font-semibold">
                          {job.job_type}
                        </div>
                        <JobTypeBadge kind={job.job_type} />
                        <div className="text-xs text-gray-400">
                          {formatDate(job.created_at)}
                        </div>
                      </div>
                      <JobStatusBadge status={job.status} />
                    </div>
                  ))
                )}
              </div>
            )}
          </MnemoSection>
          <JobDetailsModal
            open={!!selected}
            onClose={() => setSelected(null)}
            job={selected}
          />
        </section>
      </div>
    </ContentContainer>
  );
}

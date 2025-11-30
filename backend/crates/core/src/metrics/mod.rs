//! Minimal in-memory counters for Prometheus scraping.
use once_cell::sync::Lazy;
use std::sync::atomic::{AtomicU64, Ordering};

pub static INGEST_DOCS: Lazy<AtomicU64> = Lazy::new(|| AtomicU64::new(0));
pub static INGEST_CHUNKS: Lazy<AtomicU64> = Lazy::new(|| AtomicU64::new(0));
pub static QDRANT_LATENCY_MS: Lazy<AtomicU64> = Lazy::new(|| AtomicU64::new(0));
pub static SURREAL_LATENCY_MS: Lazy<AtomicU64> = Lazy::new(|| AtomicU64::new(0));
pub static RAG_QUERIES: Lazy<AtomicU64> = Lazy::new(|| AtomicU64::new(0));
pub static RAG_CACHE_HITS: Lazy<AtomicU64> = Lazy::new(|| AtomicU64::new(0));
pub static JOBS_RUNNING: Lazy<AtomicU64> = Lazy::new(|| AtomicU64::new(0));

pub fn inc_ingest_docs(v: u64) {
    INGEST_DOCS.fetch_add(v, Ordering::Relaxed);
}

pub fn inc_ingest_chunks(v: u64) {
    INGEST_CHUNKS.fetch_add(v, Ordering::Relaxed);
}

pub fn record_qdrant_latency(ms: u64) {
    QDRANT_LATENCY_MS.store(ms, Ordering::Relaxed);
}

pub fn record_surreal_latency(ms: u64) {
    SURREAL_LATENCY_MS.store(ms, Ordering::Relaxed);
}

pub fn inc_rag_queries() {
    RAG_QUERIES.fetch_add(1, Ordering::Relaxed);
}

pub fn inc_rag_cache_hits() {
    RAG_CACHE_HITS.fetch_add(1, Ordering::Relaxed);
}

pub fn set_jobs_running(count: u64) {
    JOBS_RUNNING.store(count, Ordering::Relaxed);
}

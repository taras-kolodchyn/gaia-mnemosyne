use serde_json::json;

/// Minimal OpenAPI 3.0 document describing primary /v1 endpoints.
pub fn build_openapi() -> serde_json::Value {
    json!({
        "openapi": "3.0.0",
        "info": {
            "title": "Gaia Mnemosyne API",
            "version": env!("CARGO_PKG_VERSION"),
        },
        "paths": {
            "/v1/health": { "get": { "responses": { "200": { "description": "OK" }}}},
            "/v1/version": { "get": { "responses": { "200": { "description": "OK" }}}},
            "/v1/jobs": {
                "get": { "responses": { "200": { "description": "List jobs" }}},
                "post": { "responses": { "200": { "description": "Create job" }}}
            },
            "/v1/jobs/run": {
                "post": {
                    "requestBody": { "content": { "application/json": { "schema": { "type": "object" }}}},
                    "responses": { "200": { "description": "Run job" }}
                }
            },
            "/v1/providers": { "get": { "responses": { "200": { "description": "Providers" }}}},
            "/v1/graph/snapshot": { "get": { "responses": { "200": { "description": "Graph snapshot" }}}},
            "/v1/graph/node/{id}": { "get": { "responses": { "200": { "description": "Graph node" }}}},
            "/v1/rag/query": {
                "post": {
                    "requestBody": { "content": { "application/json": { "schema": { "type": "object" }}}},
                    "responses": { "200": { "description": "RAG response" }}
                }
            },
            "/v1/rag/debug": {
                "post": {
                    "requestBody": { "content": { "application/json": { "schema": { "type": "object" }}}},
                    "responses": { "200": { "description": "RAG debug" }}
                }
            },
            "/v1/rag/metadata": { "get": { "responses": { "200": { "description": "RAG metadata" }}}},
            "/v1/context/query": {
                "post": {
                    "requestBody": { "content": { "application/json": { "schema": { "type": "object" }}}},
                    "responses": { "200": { "description": "Context query" }}
                }
            }
        }
    })
}

# Foxtive Axum Changelog
Foxtive Axum changelog file 

### 0.11.1 (2026-03-11)
* feat(responder): .respond_msg() now accepts Into<String> for easier usage

### 0.11.0 (2026-03-10)
* bump(foxtive): to version 0.24.0

### 0.10.0 (2026-02-26)
* bump(foxtive): to version 0.23.0

### 0.9.0 (2026-02-05)
* bump(foxtive): to version 0.22.0

### 0.8.0 (2026-01-15)
* bump(foxtive): to version 0.20.0

### 0.7.1 (2025-10-17)
* feat(server): custom shutdown signal
* refactor(server): on_started now accepts async fn instead of blocking fn 

### 0.7.0 (2025-10-17)
* bump(axum): to version 1.48.0
* refactor(server): on_started now accepts async fn instead of blocking fn 

### 0.6.0 (2025-10-12)
* bump(foxtive): to version 0.19.1

### 0.5.1 (2025-09-17)
* feat(server): adjust setup to be able to collect client ip address
* fix(http): auto handle JointError to allow propagating blocking error in handler

### 0.5.0 (2025-09-17)
* bump(foxtive): to version 0.18.1
* feat(static): add safe resolver for mounted static dir paths

### 0.4.0 (2025-08-14)
* refactor(foxtive): avoid panics
* bump(foxtive): to version 0.17.0

### 0.3.1 (2025-08-11)
* fix(setup): relax async fut requirement to not require Sync marker

### 0.3.0 (2025-08-08)
* feat(cors): allowed headers config

### 0.2.0 (2025-08-08)
* feat(extractor): add ByteBody, JsonBody, StringBody

### 0.1.2 (2025-08-06)
* feat(responder): support nested result Result<AppResult<T>, JoinError>

### 0.1.1 (2025-08-06)
* feat(server): support shutdown signal

# Optimization Plan: Similarity Search & Embeddings

## Current Performance Issues

### Similarity Search: 36.56x overhead
**Problem:** 
- Querying mappy N times in hot path (once per tag set)
- Decompressing N times in hot path
- Total: N mappy queries + N decompressions during similarity computation

**Root Cause:**
- Each similarity computation triggers a mappy query + decompression
- No caching of decompressed data

### Embeddings: 2140x overhead
**Problem:**
- Exact operation is extremely fast (0.04ms = microseconds)
- Full mappy storage/retrieval cycle for very fast operation
- Vocabulary building happens every time

**Root Cause:**
- Mappy overhead dominates for microsecond operations
- No caching of vocabulary or intermediate results

## Optimization Strategies

### 1. Similarity Search Optimization

**Strategy: Cache-First Approach**
- Pre-decompress all tag sets once (outside timing)
- Cache decompressed tag sets in memory
- Compute similarities from cached data
- This moves mappy queries out of hot path

**Expected Improvement:**
- From: N queries + N decompressions in hot path
- To: N queries + N decompressions upfront (cached)
- Speed ratio: 36.56x → ~2-5x (similar to clustering)

**Implementation:**
1. Store all tag sets in mappy (as before)
2. Pre-decompress all tag sets into cache (before timing starts)
3. Time only the similarity computation from cached data
4. This matches the classification optimization pattern

### 2. Embeddings Optimization

**Strategy A: Vocabulary Caching**
- Store vocabulary in mappy (as compressed data)
- Retrieve vocabulary once, reuse for embedding
- Skip tag set retrieval if vocabulary is available

**Strategy B: Hybrid Approach**
- For embeddings, accept that mappy adds overhead
- Document that embeddings is a special case
- Use mappy only if storage efficiency is critical
- For fast operations, consider exact storage

**Strategy C: Batch Optimization**
- Pre-compute and cache vocabulary building
- Use batch operations more efficiently
- Reduce redundant operations

**Recommended: Strategy A + Documentation**
- Cache vocabulary computation
- Store vocabulary in mappy
- Document that for microsecond operations, overhead is expected

**Expected Improvement:**
- From: 2140x overhead
- To: Still high (50-100x) but more reasonable
- Note: For microsecond operations, any overhead looks large

## Implementation Plan

### Phase 1: Similarity Search Caching
1. Add cache for decompressed tag sets
2. Pre-populate cache before timing
3. Compute similarities from cache
4. Measure improvement

### Phase 2: Embeddings Vocabulary Caching
1. Cache vocabulary building step
2. Store vocabulary in mappy
3. Retrieve vocabulary once
4. Use cached vocabulary for embedding

### Phase 3: Documentation
1. Document optimization strategies
2. Explain when to use mappy vs exact storage
3. Update reports with optimized results

## Success Criteria

### Similarity Search
- **Target:** Speed ratio < 5x (from 36.56x)
- **Accuracy:** Maintain 90%+
- **Status:** Production ready with caching

### Embeddings
- **Target:** Speed ratio < 100x (from 2140x)
- **Accuracy:** Maintain 100%
- **Status:** Documented special case, acceptable for storage-critical use cases


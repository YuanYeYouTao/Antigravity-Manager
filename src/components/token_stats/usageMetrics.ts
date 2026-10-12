export interface CacheReportedStats {
    cache_reported_requests: number;
    cache_reported_input_tokens: number;
    cache_reported_tokens: number;
}

export interface CacheMetrics {
    hitRate: number | null;
    coverageRate: number | null;
    reportedRequests: number;
    totalRequests: number;
    cachedTokens: number | null;
    uncachedInputTokens: number | null;
}

export interface ModelPricingRule {
    input: number;
    output: number;
    cached: number;
}

export interface NominalCosts {
    input: number | null;
    output: number | null;
}

export interface NominalCostTotals extends NominalCosts {
    pricedModels: number;
    totalModels: number;
}

export function cacheMetricsFor(stats: CacheReportedStats | null, requests: number): CacheMetrics {
    const reported = stats?.cache_reported_requests ?? 0;
    return {
        hitRate: stats && stats.cache_reported_input_tokens > 0
            ? stats.cache_reported_tokens / stats.cache_reported_input_tokens * 100
            : null,
        coverageRate: requests > 0 ? reported / requests * 100 : null,
        reportedRequests: reported,
        totalRequests: requests,
        cachedTokens: stats && reported > 0 ? stats.cache_reported_tokens : null,
        uncachedInputTokens: stats && reported > 0
            ? Math.max(0, stats.cache_reported_input_tokens - stats.cache_reported_tokens)
            : null,
    };
}

export function nominalCostsFor(
    stats: CacheReportedStats & { total_output_tokens: number; output_reported_requests: number },
    pricing: ModelPricingRule | null,
): NominalCosts {
    if (pricing === null) return { input: null, output: null };
    return {
        input: stats.cache_reported_requests > 0
            ? (Math.max(0, stats.cache_reported_input_tokens - stats.cache_reported_tokens) * pricing.input
                + stats.cache_reported_tokens * pricing.cached) / 1_000_000
            : null,
        output: stats.output_reported_requests > 0 ? stats.total_output_tokens * pricing.output / 1_000_000 : null,
    };
}

export function sumNominalCosts(models: NominalCosts[], pricedModels: number): NominalCostTotals {
    const inputs = models.flatMap(model => model.input === null ? [] : [model.input]);
    const outputs = models.flatMap(model => model.output === null ? [] : [model.output]);
    return {
        input: inputs.length > 0 ? inputs.reduce((sum, value) => sum + value, 0) : null,
        output: outputs.length > 0 ? outputs.reduce((sum, value) => sum + value, 0) : null,
        pricedModels,
        totalModels: models.length,
    };
}

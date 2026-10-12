import { useTranslation } from 'react-i18next';
import type { CacheMetrics } from './usageMetrics';

export function CacheCoverage({ metrics }: { metrics: CacheMetrics }) {
    const { t } = useTranslation();
    return (
        <div className="text-[11px] text-gray-500 dark:text-gray-400 space-y-0.5">
            <div>
                {t('token_stats.known_cache_hit_rate', '已知样本命中率')}: {' '}
                <span className="font-mono">
                    {metrics.hitRate === null
                        ? t('token_stats.cache_not_reported', '未返回')
                        : `${metrics.hitRate.toFixed(1)}%`}
                </span>
            </div>
            <div>
                {t('token_stats.cache_field_coverage', '缓存字段覆盖')}: {' '}
                <span className="font-mono">
                    {metrics.reportedRequests}/{metrics.totalRequests}
                    {metrics.coverageRate === null ? '' : ` (${metrics.coverageRate.toFixed(1)}%)`}
                </span>
            </div>
        </div>
    );
}

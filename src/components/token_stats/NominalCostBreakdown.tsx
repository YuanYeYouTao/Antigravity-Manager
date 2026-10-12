import { useTranslation } from 'react-i18next';
import type { NominalCostTotals } from './usageMetrics';

export function NominalCostBreakdown({ costs, formatCurrency }: {
    costs: NominalCostTotals;
    formatCurrency: (value: number) => string;
}) {
    const { t } = useTranslation();
    const missingPrice = t('token_stats.price_not_configured', '未配置单价');
    return (
        <div className="text-xs space-y-1">
            <div className="flex justify-between gap-2">
                <span>{t('token_stats.reported_cache_input_cost', '已知样本输入')}</span>
                <span className="font-mono text-emerald-600 dark:text-emerald-400">
                    {costs.input === null
                        ? costs.pricedModels === 0 ? missingPrice : t('token_stats.cache_not_reported', '未返回')
                        : formatCurrency(costs.input)}
                </span>
            </div>
            <div className="flex justify-between gap-2">
                <span>{t('token_stats.reported_output_cost', '已报告输出')}</span>
                <span className="font-mono text-emerald-600 dark:text-emerald-400">
                    {costs.output === null ? costs.pricedModels === 0 ? missingPrice : t('token_stats.cache_not_reported', '未返回') : formatCurrency(costs.output)}
                </span>
            </div>
            <div className="text-[10px] text-gray-500 dark:text-gray-400">
                {t('token_stats.priced_model_coverage', '已配置单价 {{known}}/{{total}} 模型', {
                    known: costs.pricedModels, total: costs.totalModels,
                })}
            </div>
            <div className="text-[10px] text-gray-500 dark:text-gray-400">
                {t('token_stats.partial_nominal_cost_note', '仅按已报告用量估算，不代表完整费用')}
            </div>
        </div>
    );
}

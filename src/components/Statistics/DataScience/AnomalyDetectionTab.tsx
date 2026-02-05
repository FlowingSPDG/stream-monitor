import { useQuery } from '@tanstack/react-query';
import { Scatter, ScatterChart, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer, Legend, ReferenceLine, Cell } from 'recharts';
import { StatCardSkeleton, ChartSkeleton } from '../../common/Skeleton';
import { detectAnomalies } from '../../../api/statistics';

interface AnomalyDetectionTabProps {
  channelId: number | null;
  startTime: string;
  endTime: string;
}

const AnomalyDetectionTab = ({ channelId, startTime, endTime }: AnomalyDetectionTabProps) => {
  // チャンネル選択チェック
  if (channelId === null) {
    return (
      <div className="bg-yellow-50 dark:bg-yellow-900/20 border border-yellow-200 dark:border-yellow-800 rounded-lg p-6">
        <div className="flex items-start">
          <div className="flex-shrink-0">
            <svg className="h-6 w-6 text-yellow-600 dark:text-yellow-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
            </svg>
          </div>
          <div className="ml-3">
            <h3 className="text-sm font-medium text-yellow-800 dark:text-yellow-200">
              チャンネルを選択してください
            </h3>
            <p className="mt-2 text-sm text-yellow-700 dark:text-yellow-300">
              異常検知には特定のチャンネルを選択する必要があります。上部のチャンネル選択ドロップダウンから分析対象のチャンネルを選んでください。
            </p>
          </div>
        </div>
      </div>
    );
  }

  const { data, isLoading } = useQuery({
    queryKey: ['anomalyDetection', channelId, startTime, endTime],
    queryFn: () => detectAnomalies({
      channelId,
      streamId: null,
      startTime,
      endTime,
      zThreshold: 3.0, // 検出感度: 3.0 = 厳格（IQR multiplier 2.0）
    }),
    enabled: !!channelId,
  });

  if (isLoading) {
    return (
      <div className="space-y-6">
        <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
          <StatCardSkeleton />
          <StatCardSkeleton />
        </div>
        <ChartSkeleton height={400} />
      </div>
    );
  }

  if (!data) {
    return (
      <div className="bg-yellow-50 dark:bg-yellow-900/20 border border-yellow-200 dark:border-yellow-800 rounded-lg p-4">
        <p className="text-yellow-800 dark:text-yellow-200">データがありません</p>
      </div>
    );
  }

  // フィルター: 1971年以前の不正なタイムスタンプを除外
  const validViewerAnomalies = data.viewerAnomalies.filter(a => {
    const date = new Date(a.timestamp);
    const time = date.getTime();
    // NaN、Invalid Date、または1971年以前を除外
    return !isNaN(time) && time > new Date('1971-01-01').getTime();
  });

  const validChatAnomalies = data.chatAnomalies.filter(a => {
    const date = new Date(a.timestamp);
    const time = date.getTime();
    // NaN、Invalid Date、または1971年以前を除外
    return !isNaN(time) && time > new Date('1971-01-01').getTime();
  });

  const getTrendIcon = (trend: string) => {
    switch (trend) {
      case 'increasing':
        return '📈';
      case 'decreasing':
        return '📉';
      case 'stable':
        return '➡️';
      default:
        return '❓';
    }
  };

  const getTrendColor = (trend: string) => {
    switch (trend) {
      case 'increasing':
        return 'text-green-600 dark:text-green-400';
      case 'decreasing':
        return 'text-red-600 dark:text-red-400';
      case 'stable':
        return 'text-blue-600 dark:text-blue-400';
      default:
        return 'text-gray-600 dark:text-gray-400';
    }
  };

  return (
    <div className="space-y-8">
      {/* Trend Summary */}
      <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
        {/* Viewer Trend */}
        <div className="bg-white dark:bg-gray-800 rounded-lg p-6 shadow">
          <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-4">視聴者トレンド</h3>
          <div className="flex items-center justify-between mb-4">
            <div className={`text-4xl font-bold ${getTrendColor(data.trendStats.viewerTrend)}`}>
              {getTrendIcon(data.trendStats.viewerTrend)} {data.trendStats.viewerTrend}
            </div>
          </div>
          <div className="space-y-2 text-sm">
            <div className="flex justify-between">
              <span className="text-gray-600 dark:text-gray-400">中央値:</span>
              <span className="font-medium text-gray-900 dark:text-gray-100">
                {Math.round(data.trendStats.viewerMedian).toLocaleString()}
              </span>
            </div>
            <div className="flex justify-between">
              <span className="text-gray-600 dark:text-gray-400">MAD:</span>
              <span className="font-medium text-gray-900 dark:text-gray-100">
                {Math.round(data.trendStats.viewerMad).toLocaleString()}
              </span>
            </div>
            <div className="flex justify-between">
              <span className="text-gray-600 dark:text-gray-400">平均値:</span>
              <span className="font-medium text-gray-900 dark:text-gray-100">
                {Math.round(data.trendStats.viewerAvg).toLocaleString()}
              </span>
            </div>
            <div className="flex justify-between">
              <span className="text-gray-600 dark:text-gray-400">検出異常数:</span>
              <span className="font-medium text-gray-900 dark:text-gray-100">
                {validViewerAnomalies.length}
              </span>
            </div>
          </div>
        </div>

        {/* Chat Trend */}
        <div className="bg-white dark:bg-gray-800 rounded-lg p-6 shadow">
          <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-4">チャットトレンド</h3>
          <div className="flex items-center justify-between mb-4">
            <div className={`text-4xl font-bold ${getTrendColor(data.trendStats.chatTrend)}`}>
              {getTrendIcon(data.trendStats.chatTrend)} {data.trendStats.chatTrend}
            </div>
          </div>
          <div className="space-y-2 text-sm">
            <div className="flex justify-between">
              <span className="text-gray-600 dark:text-gray-400">平均値:</span>
              <span className="font-medium text-gray-900 dark:text-gray-100">
                {Math.round(data.trendStats.chatAvg).toLocaleString()}
              </span>
            </div>
            <div className="flex justify-between">
              <span className="text-gray-600 dark:text-gray-400">標準偏差:</span>
              <span className="font-medium text-gray-900 dark:text-gray-100">
                {Math.round(data.trendStats.chatStdDev).toLocaleString()}
              </span>
            </div>
            <div className="flex justify-between">
              <span className="text-gray-600 dark:text-gray-400">検出異常数:</span>
              <span className="font-medium text-gray-900 dark:text-gray-100">
                {data.chatAnomalies.length}
              </span>
            </div>
          </div>
        </div>
      </div>

      {/* Viewer Anomalies */}
      {validViewerAnomalies.length > 0 && (
        <div className="bg-white dark:bg-gray-800 rounded-lg p-6 shadow">
          <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-4">視聴者数の異常値</h3>
          <p className="text-sm text-gray-600 dark:text-gray-400 mb-4">
            Modified Z-Score（MADベース）を使用して統計的に有意な異常を検出します。
            Twitchの更新間隔を考慮し、連続する同じ値は除外されています。
            緑色はスパイク（急増）、赤色はドロップ（急減）を示します。
          </p>

          {/* Anomalies Scatter Plot */}
          <div className="mb-6">
            <ResponsiveContainer width="100%" height={350}>
              <ScatterChart margin={{ top: 20, right: 20, bottom: 60, left: 60 }}>
                <CartesianGrid strokeDasharray="3 3" stroke="#374151" />
                <XAxis
                  type="number"
                  dataKey="timestampMs"
                  name="時刻"
                  domain={['dataMin', 'dataMax']}
                  stroke="#9ca3af"
                  tickFormatter={(ts) => {
                    const date = new Date(ts);
                    return `${date.getMonth() + 1}/${date.getDate()} ${date.getHours()}:${String(date.getMinutes()).padStart(2, '0')}`;
                  }}
                  label={{ value: '日時', position: 'insideBottom', offset: -10, fill: '#9ca3af' }}
                />
                <YAxis
                  type="number"
                  dataKey="value"
                  name="視聴者数"
                  stroke="#9ca3af"
                  label={{ value: '視聴者数', angle: -90, position: 'insideLeft', fill: '#9ca3af' }}
                />
                <Tooltip
                  cursor={{ strokeDasharray: '3 3' }}
                  contentStyle={{
                    backgroundColor: 'rgba(31, 41, 55, 0.9)',
                    border: '1px solid #4b5563',
                    borderRadius: '0.375rem',
                  }}
                  labelStyle={{ color: '#f3f4f6' }}
                  itemStyle={{ color: '#f3f4f6' }}
                  formatter={(value: any, name?: string) => {
                    if (name === 'value') return [Math.round(value).toLocaleString(), '現在の視聴者数'];
                    if (name === 'previousValue') return [Math.round(value).toLocaleString(), '直前の視聴者数'];
                    if (name === 'changeAmount') return [
                      `${value > 0 ? '+' : ''}${Math.round(value).toLocaleString()}`,
                      '変化量'
                    ];
                    if (name === 'changeRate') return [
                      `${value > 0 ? '+' : ''}${value.toFixed(1)}%`,
                      '変化率'
                    ];
                    if (name === 'modifiedZScore') return [value.toFixed(2), 'Modified Z-Score'];
                    return [value, name || ''];
                  }}
                  labelFormatter={(label: any) => {
                    const date = new Date(label);
                    return `${date.getFullYear()}/${date.getMonth() + 1}/${date.getDate()} ${date.getHours()}:${String(date.getMinutes()).padStart(2, '0')}:${String(date.getSeconds()).padStart(2, '0')}`;
                  }}
                />
                <Legend />
                <ReferenceLine y={data.trendStats.viewerAvg} stroke="#3b82f6" strokeDasharray="3 3" />
                <Scatter
                  name="異常値"
                  data={validViewerAnomalies.map((a) => ({
                    timestampMs: new Date(a.timestamp).getTime(),
                    value: a.value,
                    previousValue: a.previousValue,
                    changeAmount: a.changeAmount,
                    changeRate: a.changeRate,
                    modifiedZScore: a.modifiedZScore,
                  }))}
                  fill="#ef4444"
                >
                  {validViewerAnomalies.map((a, index) => (
                    <Cell key={index} fill={a.isPositive ? '#10b981' : '#ef4444'} />
                  ))}
                </Scatter>
              </ScatterChart>
            </ResponsiveContainer>
          </div>

          {/* Anomalies Table */}
          <div className="overflow-x-auto">
            <table className="min-w-full divide-y divide-gray-200 dark:divide-gray-700">
              <thead className="bg-gray-50 dark:bg-gray-900">
                <tr>
                  <th className="px-4 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase">
                    日時
                  </th>
                  <th className="px-4 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase">
                    配信経過
                  </th>
                  <th className="px-4 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase">
                    直前 → 現在
                  </th>
                  <th className="px-4 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase">
                    変化量
                  </th>
                  <th className="px-4 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase">
                    変化率
                  </th>
                  <th className="px-4 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase">
                    M-Z Score
                  </th>
                  <th className="px-4 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase">
                    タイプ
                  </th>
                </tr>
              </thead>
              <tbody className="bg-white dark:bg-gray-800 divide-y divide-gray-200 dark:divide-gray-700">
                {validViewerAnomalies.map((anomaly, idx) => {
                  const getStreamPhaseLabel = (phase: string) => {
                    switch (phase) {
                      case 'early': return '序盤';
                      case 'mid': return '中盤';
                      case 'late': return '終盤';
                      default: return '不明';
                    }
                  };

                  const getStreamPhaseColor = (phase: string) => {
                    switch (phase) {
                      case 'early': return 'bg-blue-100 text-blue-800 dark:bg-blue-900/30 dark:text-blue-300';
                      case 'mid': return 'bg-purple-100 text-purple-800 dark:bg-purple-900/30 dark:text-purple-300';
                      case 'late': return 'bg-orange-100 text-orange-800 dark:bg-orange-900/30 dark:text-orange-300';
                      default: return 'bg-gray-100 text-gray-800 dark:bg-gray-900/30 dark:text-gray-300';
                    }
                  };

                  const formatStreamTime = (minutes?: number) => {
                    if (minutes === undefined) return '不明';
                    const hours = Math.floor(minutes / 60);
                    const mins = minutes % 60;
                    if (hours > 0) {
                      return `+${hours}時間${mins}分`;
                    }
                    return `+${mins}分`;
                  };

                  return (
                    <tr key={idx} className="hover:bg-gray-50 dark:hover:bg-gray-700">
                      <td className="px-4 py-3 whitespace-nowrap text-sm text-gray-900 dark:text-gray-100">
                        {new Date(anomaly.timestamp).toLocaleString('ja-JP')}
                      </td>
                      <td className="px-4 py-3 whitespace-nowrap text-sm">
                        <div className="flex flex-col gap-1">
                          <span className="font-medium text-gray-900 dark:text-gray-100">
                            {formatStreamTime(anomaly.minutesFromStreamStart)}
                          </span>
                          <span className={`px-2 py-0.5 rounded text-xs inline-block w-fit ${getStreamPhaseColor(anomaly.streamPhase)}`}>
                            {getStreamPhaseLabel(anomaly.streamPhase)}
                          </span>
                        </div>
                      </td>
                      <td className="px-4 py-3 whitespace-nowrap text-sm font-medium text-gray-900 dark:text-gray-100">
                        <span className="text-gray-500">{Math.round(anomaly.previousValue).toLocaleString()}</span>
                        {' → '}
                        <span className={anomaly.isPositive ? 'text-green-600 dark:text-green-400' : 'text-red-600 dark:text-red-400'}>
                          {Math.round(anomaly.value).toLocaleString()}
                        </span>
                      </td>
                      <td className={`px-4 py-3 whitespace-nowrap text-sm font-bold ${
                        anomaly.isPositive ? 'text-green-600 dark:text-green-400' : 'text-red-600 dark:text-red-400'
                      }`}>
                        {anomaly.changeAmount > 0 ? '+' : ''}{Math.round(anomaly.changeAmount).toLocaleString()}
                      </td>
                      <td className={`px-4 py-3 whitespace-nowrap text-sm font-bold ${
                        anomaly.isPositive ? 'text-green-600 dark:text-green-400' : 'text-red-600 dark:text-red-400'
                      }`}>
                        {anomaly.changeRate > 0 ? '+' : ''}{anomaly.changeRate.toFixed(1)}%
                      </td>
                      <td className={`px-4 py-3 whitespace-nowrap text-sm font-bold ${
                        anomaly.isPositive ? 'text-green-600 dark:text-green-400' : 'text-red-600 dark:text-red-400'
                      }`}>
                        {anomaly.modifiedZScore.toFixed(2)}
                      </td>
                      <td className="px-4 py-3 whitespace-nowrap text-sm">
                        <span className={`px-2 py-1 rounded text-xs ${
                          anomaly.isPositive
                            ? 'bg-green-100 text-green-800 dark:bg-green-900/30 dark:text-green-300'
                            : 'bg-red-100 text-red-800 dark:bg-red-900/30 dark:text-red-300'
                        }`}>
                          {anomaly.isPositive ? '🚀 スパイク' : '📉 ドロップ'}
                        </span>
                      </td>
                    </tr>
                  );
                })}
              </tbody>
            </table>
          </div>
        </div>
      )}

      {/* No Anomalies Message */}
      {validViewerAnomalies.length === 0 && (
        <div className="bg-green-50 dark:bg-green-900/20 border border-green-200 dark:border-green-800 rounded-lg p-4">
          <p className="text-green-800 dark:text-green-200">
            ✅ 選択した期間に大きな異常値は検出されませんでした。データは安定しています。
          </p>
        </div>
      )}

      {/* Explanation */}
      <div className="bg-indigo-50 dark:bg-indigo-900/20 border border-indigo-200 dark:border-indigo-800 rounded-lg p-6">
        <h3 className="text-lg font-semibold text-indigo-900 dark:text-indigo-100 mb-3">
          異常検知について
        </h3>
        <div className="space-y-3 text-sm text-indigo-800 dark:text-indigo-200">
          <div>
            <p className="font-medium mb-1">📊 Modified Z-Score（MADベース）:</p>
            <p className="ml-4">
              統計的に頑健な異常検知手法です。中央値とMAD（中央絶対偏差）を使用するため、
              外れ値の影響を受けず、小規模・大規模配信問わず同じ基準で異常を検出できます。
            </p>
          </div>
          <div>
            <p className="font-medium mb-1">🔄 Twitch仕様への対応:</p>
            <p className="ml-4">
              Twitchの視聴者数は完全なリアルタイムではなく、数分間隔で更新されます。
              連続する同じ値を除去し、実際に値が変化したポイントのみを分析対象とすることで、
              プラットフォームの正常な更新を異常として誤検出することを防ぎます。
            </p>
          </div>
          <div>
            <p className="font-medium mb-1">📈 統計指標の意味:</p>
            <ul className="ml-4 space-y-1">
              <li>• <strong>中央値:</strong> データを大きさ順に並べたときの中央の値（平均値より頑健）</li>
              <li>• <strong>MAD:</strong> 中央値からの偏差の中央値（標準偏差より頑健）</li>
              <li>• <strong>Modified Z-Score:</strong> 0.6745 × (値 - 中央値) / MAD で計算</li>
              <li>• <strong>閾値:</strong> |Modified Z-Score| ≥ 3.0 で異常と判定（デフォルト）</li>
            </ul>
          </div>
          <div>
            <p className="font-medium mb-1">🔍 検出される異常の種類:</p>
            <ul className="ml-4 space-y-1">
              <li>• <strong>スパイク（正の異常）:</strong> 視聴者数が統計的に有意に増加したポイント</li>
              <li>• <strong>ドロップ（負の異常）:</strong> 視聴者数が統計的に有意に減少したポイント</li>
            </ul>
          </div>
          <div>
            <p className="font-medium mb-1">⚙️ 検出の仕組み:</p>
            <ul className="ml-4 space-y-1">
              <li>• 連続する同じ視聴者数を1つにまとめる（Twitch更新間隔の考慮）</li>
              <li>• 実際に値が変化したポイントのみで中央値とMADを計算</li>
              <li>• 各変化ポイントのModified Z-Scoreを計算</li>
              <li>• 閾値（デフォルト3.0）を超えるポイントを異常として検出</li>
              <li>• 短時間（5分未満）での変化は1.5倍厳格な閾値を適用</li>
              <li>• <strong>配信の最初10%のみ除外</strong>（配信開始時の自然な増加を誤検出防止）</li>
              <li>• 終盤のスパイク/ドロップは検出対象（終了間際の重要イベントを見逃さない）</li>
              <li>• 視聴者数0のポイントは事前に除外</li>
              <li>• 各異常に「配信開始からの経過時間」と「配信フェーズ」タグを表示</li>
              <li>• 最も顕著な上位50件のみを表示</li>
            </ul>
          </div>
          <div>
            <p className="font-medium mb-1">🏷️ 配信フェーズ（配信全体に対する相対位置）:</p>
            <ul className="ml-4 space-y-1">
              <li>• <strong className="text-blue-600 dark:text-blue-400">序盤</strong>: 配信の最初の1/3（0～33%）</li>
              <li>• <strong className="text-purple-600 dark:text-purple-400">中盤</strong>: 配信の中間1/3（33～67%）</li>
              <li>• <strong className="text-orange-600 dark:text-orange-400">終盤</strong>: 配信の最後の1/3（67～100%）</li>
              <li className="text-gray-600 dark:text-gray-400 text-xs mt-2">※ 10分配信でも12時間配信でも、同じ基準で判定されます</li>
            </ul>
          </div>
          <div>
            <p className="font-medium mb-1">🎯 規模に依存しない検出:</p>
            <p className="ml-4">
              小規模配信（50人規模）でも大規模配信（10,000人規模）でも、同じ統計的基準で異常を検出します。
              例: 50人→100人（+100%）と5,000人→10,000人（+100%）は、どちらも同等の異常度として評価されます。
            </p>
          </div>
          <div>
            <p className="font-medium mb-1">🔗 トレンドとの相関:</p>
            <p className="ml-4">
              視聴者トレンドも同じMADベースの統計手法を使用しており、
              前半と後半の中央値の差がMADの1.5倍以上で「増加」または「減少」トレンドと判定されます。
            </p>
          </div>
          <div>
            <p className="font-medium mb-1">💡 活用方法:</p>
            <ul className="ml-4 space-y-1">
              <li>• 変化量と変化率から、異常の規模を定量的に評価</li>
              <li>• Modified Z-Scoreが高いほど統計的に稀な現象（通常3.0～10.0程度）</li>
              <li>• スパイク発生時の配信内容やSNS投稿を分析してバイラル要因を特定</li>
              <li>• ドロップ発生時の状況（技術的問題、内容変更など）を確認</li>
            </ul>
          </div>
        </div>
      </div>
    </div>
  );
};

export default AnomalyDetectionTab;

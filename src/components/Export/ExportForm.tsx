import { Channel } from "../../types";

type ExportFormat = 'csv' | 'json' | 'parquet';
type AggregationType = 'raw' | '1min' | '5min' | '1hour';

interface ExportConfig {
  channelIds: number[];
  startDate: string;
  endDate: string;
  format: ExportFormat;
  aggregation: AggregationType;
  includeChatData: boolean;
}

interface ExportFormProps {
  config: ExportConfig;
  onConfigChange: (config: ExportConfig) => void;
  channels: Channel[];
}

export function ExportForm({ config, onConfigChange, channels }: ExportFormProps) {
  const updateConfig = (updates: Partial<ExportConfig>) => {
    onConfigChange({ ...config, ...updates });
  };

  const handleChannelToggle = (channelId: number, checked: boolean) => {
    if (checked) {
      updateConfig({ channelIds: [...config.channelIds, channelId] });
    } else {
      updateConfig({ channelIds: config.channelIds.filter(id => id !== channelId) });
    }
  };

  const handleSelectAllChannels = () => {
    updateConfig({ channelIds: channels.map(c => c.id!).filter(Boolean) });
  };

  const handleDeselectAllChannels = () => {
    updateConfig({ channelIds: [] });
  };

  const formatOptions = [
    { value: 'csv' as const, label: 'CSV', description: '表計算ソフト対応' },
    { value: 'json' as const, label: 'JSON', description: '構造化データ' },
    { value: 'parquet' as const, label: 'Parquet', description: 'ビッグデータ対応' },
  ];

  const aggregationOptions = [
    { value: 'raw' as const, label: '生データ', description: '全ての収集データをそのまま' },
    { value: '1min' as const, label: '1分集計', description: '1分単位で平均化' },
    { value: '5min' as const, label: '5分集計', description: '5分単位で平均化' },
    { value: '1hour' as const, label: '1時間集計', description: '1時間単位で平均化' },
  ];

  return (
    <div className="space-y-6">
      <h2 className="text-xl font-semibold text-gray-900">エクスポート設定</h2>

      {/* チャンネル選択 */}
      <div>
        <label className="block text-sm font-medium text-gray-700 mb-3">
          エクスポート対象チャンネル
        </label>

        <div className="flex space-x-4 mb-3">
          <button
            onClick={handleSelectAllChannels}
            className="px-3 py-1 text-sm bg-blue-100 hover:bg-blue-200 text-blue-800 rounded-md"
          >
            全て選択
          </button>
          <button
            onClick={handleDeselectAllChannels}
            className="px-3 py-1 text-sm bg-gray-100 hover:bg-gray-200 text-gray-800 rounded-md"
          >
            全て解除
          </button>
        </div>

        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-3 max-h-48 overflow-y-auto border border-gray-300 rounded-md p-3">
          {channels.length === 0 ? (
            <p className="text-gray-500 text-sm">チャンネルが登録されていません</p>
          ) : (
            channels.map((channel) => (
              <label key={channel.id} className="flex items-center space-x-2">
                <input
                  type="checkbox"
                  checked={config.channelIds.includes(channel.id!)}
                  onChange={(e) => handleChannelToggle(channel.id!, e.target.checked)}
                  className="rounded border-gray-300 text-blue-600 focus:ring-blue-500"
                />
                <span className="text-sm text-gray-700">
                  {channel.display_name || channel.channel_name}
                  <span className="text-xs text-gray-500 ml-1">
                    ({channel.platform})
                  </span>
                </span>
              </label>
            ))
          )}
        </div>
        <p className="text-xs text-gray-500 mt-1">
          選択されたチャンネル: {config.channelIds.length}個
        </p>
      </div>

      {/* 日付範囲選択 */}
      <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
        <div>
          <label className="block text-sm font-medium text-gray-700 mb-1">
            開始日
          </label>
          <input
            type="date"
            value={config.startDate}
            onChange={(e) => updateConfig({ startDate: e.target.value })}
            className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
          />
        </div>

        <div>
          <label className="block text-sm font-medium text-gray-700 mb-1">
            終了日
          </label>
          <input
            type="date"
            value={config.endDate}
            onChange={(e) => updateConfig({ endDate: e.target.value })}
            className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
          />
        </div>
      </div>

      {/* エクスポート形式選択 */}
      <div>
        <label className="block text-sm font-medium text-gray-700 mb-3">
          エクスポート形式
        </label>
        <div className="grid grid-cols-1 md:grid-cols-3 gap-3">
          {formatOptions.map((option) => (
            <label
              key={option.value}
              className={`relative flex cursor-pointer rounded-lg border p-4 focus:outline-none ${
                config.format === option.value
                  ? 'border-blue-500 bg-blue-50'
                  : 'border-gray-300 bg-white hover:bg-gray-50'
              }`}
            >
              <input
                type="radio"
                name="format"
                value={option.value}
                checked={config.format === option.value}
                onChange={(e) => updateConfig({ format: e.target.value as ExportFormat })}
                className="sr-only"
              />
              <span className="flex flex-1">
                <span className="flex flex-col">
                  <span className={`block text-sm font-medium ${
                    config.format === option.value ? 'text-blue-900' : 'text-gray-900'
                  }`}>
                    {option.label}
                  </span>
                  <span className={`block text-sm ${
                    config.format === option.value ? 'text-blue-700' : 'text-gray-500'
                  }`}>
                    {option.description}
                  </span>
                </span>
              </span>
              {config.format === option.value && (
                <span className="absolute -inset-px rounded-lg border-2 border-blue-500 pointer-events-none" />
              )}
            </label>
          ))}
        </div>
      </div>

      {/* 集計オプション */}
      <div>
        <label className="block text-sm font-medium text-gray-700 mb-3">
          データ集計
        </label>
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-3">
          {aggregationOptions.map((option) => (
            <label
              key={option.value}
              className={`relative flex cursor-pointer rounded-lg border p-3 focus:outline-none ${
                config.aggregation === option.value
                  ? 'border-green-500 bg-green-50'
                  : 'border-gray-300 bg-white hover:bg-gray-50'
              }`}
            >
              <input
                type="radio"
                name="aggregation"
                value={option.value}
                checked={config.aggregation === option.value}
                onChange={(e) => updateConfig({ aggregation: e.target.value as AggregationType })}
                className="sr-only"
              />
              <span className="flex flex-1">
                <span className="flex flex-col">
                  <span className={`block text-sm font-medium ${
                    config.aggregation === option.value ? 'text-green-900' : 'text-gray-900'
                  }`}>
                    {option.label}
                  </span>
                  <span className={`block text-xs ${
                    config.aggregation === option.value ? 'text-green-700' : 'text-gray-500'
                  }`}>
                    {option.description}
                  </span>
                </span>
              </span>
            </label>
          ))}
        </div>
      </div>

      {/* 追加オプション */}
      <div>
        <label className="flex items-center">
          <input
            type="checkbox"
            checked={config.includeChatData}
            onChange={(e) => updateConfig({ includeChatData: e.target.checked })}
            className="rounded border-gray-300 text-blue-600 focus:ring-blue-500"
          />
          <span className="ml-2 text-sm text-gray-700">
            チャットデータをエクスポートに含める
          </span>
        </label>
        <p className="text-xs text-gray-500 mt-1 ml-6">
          ※ チャットデータを含む場合、ファイルサイズが大幅に増加します
        </p>
      </div>
    </div>
  );
}
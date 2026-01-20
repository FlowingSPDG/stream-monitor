import { Channel } from "../../types";

interface ChannelItemProps {
  channel: Channel;
  onEdit: (channel: Channel) => void;
  onDelete: (channelId: number) => void;
  onToggle: (channelId: number) => void;
}

export function ChannelItem({ channel, onEdit, onDelete, onToggle }: ChannelItemProps) {
  const platformColors = {
    twitch: "bg-purple-100 text-purple-800",
    youtube: "bg-red-100 text-red-800",
  };

  const platformNames = {
    twitch: "Twitch",
    youtube: "YouTube",
  };

  return (
    <div className="bg-white rounded-lg shadow border border-gray-200 p-6">
      <div className="flex items-center justify-between">
        <div className="flex items-center space-x-4">
          {/* プラットフォームバッジ */}
          <span className={`inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium ${platformColors[channel.platform as keyof typeof platformColors]}`}>
            {platformNames[channel.platform as keyof typeof platformNames]}
          </span>

          {/* チャンネル情報 */}
          <div>
            <h3 className="text-lg font-semibold text-gray-900">
              {channel.display_name || channel.channel_name}
            </h3>
            <p className="text-sm text-gray-500">
              ID: {channel.channel_id}
            </p>
          </div>
        </div>

        {/* ステータスとアクション */}
        <div className="flex items-center space-x-4">
          {/* 監視間隔 */}
          <div className="text-sm text-gray-500">
            {channel.poll_interval}秒間隔
          </div>

          {/* 有効/無効スイッチ */}
          <label className="flex items-center">
            <input
              type="checkbox"
              checked={channel.enabled}
              onChange={() => channel.id && onToggle(channel.id)}
              className="sr-only"
            />
            <div className={`relative inline-block w-10 h-6 transition duration-200 ease-in-out rounded-full ${
              channel.enabled ? 'bg-green-500' : 'bg-gray-300'
            }`}>
              <span className={`absolute left-0 top-0 m-1 w-4 h-4 bg-white rounded-full transition-transform duration-200 ease-in-out ${
                channel.enabled ? 'translate-x-4' : 'translate-x-0'
              }`}></span>
            </div>
            <span className="ml-2 text-sm text-gray-700">
              {channel.enabled ? '有効' : '無効'}
            </span>
          </label>

          {/* アクションボタン */}
          <div className="flex space-x-2">
            <button
              onClick={() => onEdit(channel)}
              className="px-3 py-1 text-sm text-blue-600 hover:text-blue-800 hover:bg-blue-50 rounded-md transition-colors"
            >
              編集
            </button>
            <button
              onClick={() => channel.id && onDelete(channel.id)}
              className="px-3 py-1 text-sm text-red-600 hover:text-red-800 hover:bg-red-50 rounded-md transition-colors"
            >
              削除
            </button>
          </div>
        </div>
      </div>

      {/* 追加情報 */}
      <div className="mt-4 flex items-center text-xs text-gray-500 space-x-4">
        <span>
          作成日: {channel.created_at ? new Date(channel.created_at).toLocaleDateString('ja-JP') : '不明'}
        </span>
        {channel.updated_at && channel.updated_at !== channel.created_at && (
          <span>
            更新日: {new Date(channel.updated_at).toLocaleDateString('ja-JP')}
          </span>
        )}
      </div>
    </div>
  );
}
import { useForm } from "react-hook-form";
import { useMutation } from "@tanstack/react-query";
import { invoke } from "@tauri-apps/api/core";
import { Channel } from "../../types";

interface ChannelFormData {
  platform: 'twitch' | 'youtube';
  channel_id: string;
  channel_name: string;
  poll_interval: number;
}

interface ChannelFormProps {
  channel?: Channel | null;
  onSuccess: () => void;
  onCancel: () => void;
}

export function ChannelForm({ channel, onSuccess, onCancel }: ChannelFormProps) {
  const { register, handleSubmit, formState: { errors }, reset } = useForm<ChannelFormData>({
    defaultValues: channel ? {
      platform: channel.platform as 'twitch' | 'youtube',
      channel_id: channel.channel_id,
      channel_name: channel.channel_name,
      poll_interval: channel.poll_interval,
    } : {
      platform: 'twitch',
      channel_id: '',
      channel_name: '',
      poll_interval: 60,
    }
  });

  const addMutation = useMutation({
    mutationFn: async (data: ChannelFormData) => {
      return await invoke<Channel>("add_channel", {
        request: {
          platform: data.platform,
          channel_id: data.channel_id,
          channel_name: data.channel_name,
          poll_interval: data.poll_interval,
        },
      });
    },
    onSuccess: () => {
      onSuccess();
      reset();
    },
  });

  const updateMutation = useMutation({
    mutationFn: async (data: ChannelFormData) => {
      if (!channel?.id) return;
      return await invoke<Channel>("update_channel", {
        id: channel.id,
        channel_name: data.channel_name,
        poll_interval: data.poll_interval,
        enabled: channel.enabled,
      });
    },
    onSuccess: () => {
      onSuccess();
    },
  });

  const onSubmit = async (data: ChannelFormData) => {
    try {
      if (channel) {
        await updateMutation.mutateAsync(data);
      } else {
        await addMutation.mutateAsync(data);
      }
    } catch (error) {
      alert(`チャンネルの${channel ? '更新' : '追加'}に失敗しました: ` + String(error));
    }
  };

  const isLoading = addMutation.isPending || updateMutation.isPending;

  return (
    <form onSubmit={handleSubmit(onSubmit)} className="space-y-4">
      <div className="flex justify-between items-center">
        <h2 className="text-lg font-semibold text-gray-900">
          {channel ? 'チャンネルを編集' : 'チャンネルを追加'}
        </h2>
        <button
          type="button"
          onClick={onCancel}
          className="text-gray-400 hover:text-gray-600"
        >
          ✕
        </button>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
        {/* プラットフォーム選択 */}
        <div>
          <label className="block text-sm font-medium text-gray-700 mb-1">
            プラットフォーム
          </label>
          <select
            {...register("platform", { required: "プラットフォームを選択してください" })}
            disabled={!!channel} // 編集時は変更不可
            className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 disabled:bg-gray-100"
          >
            <option value="twitch">Twitch</option>
            <option value="youtube">YouTube</option>
          </select>
          {errors.platform && (
            <p className="mt-1 text-sm text-red-600">{errors.platform.message}</p>
          )}
        </div>

        {/* チャンネルID */}
        <div>
          <label className="block text-sm font-medium text-gray-700 mb-1">
            チャンネルID
            {channel && <span className="text-xs text-gray-500 ml-1">（変更不可）</span>}
          </label>
          <input
            {...register("channel_id", {
              required: "チャンネルIDを入力してください",
              pattern: {
                value: /^[a-zA-Z0-9_-]+$/,
                message: "チャンネルIDは英数字、ハイフン、アンダースコアのみ使用できます"
              }
            })}
            disabled={!!channel} // 編集時は変更不可
            type="text"
            placeholder="例: shroud, UC1234567890"
            className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 disabled:bg-gray-100"
          />
          {errors.channel_id && (
            <p className="mt-1 text-sm text-red-600">{errors.channel_id.message}</p>
          )}
        </div>

        {/* チャンネル名 */}
        <div>
          <label className="block text-sm font-medium text-gray-700 mb-1">
            表示名
          </label>
          <input
            {...register("channel_name", { required: "チャンネル名を入力してください" })}
            type="text"
            placeholder="例: Shroud, チャンネル名"
            className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
          />
          {errors.channel_name && (
            <p className="mt-1 text-sm text-red-600">{errors.channel_name.message}</p>
          )}
        </div>

        {/* ポーリング間隔 */}
        <div>
          <label className="block text-sm font-medium text-gray-700 mb-1">
            監視間隔（秒）
          </label>
          <select
            {...register("poll_interval", { valueAsNumber: true })}
            className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
          >
            <option value={30}>30秒</option>
            <option value={60}>1分</option>
            <option value={180}>3分</option>
            <option value={300}>5分</option>
            <option value={600}>10分</option>
          </select>
        </div>
      </div>

      {/* ヘルプテキスト */}
      <div className="bg-blue-50 border border-blue-200 rounded-md p-3">
        <div className="text-sm text-blue-800">
          <strong>注意:</strong> プラットフォームとチャンネルIDは後から変更できません。
          正しい情報を入力してください。
        </div>
      </div>

      {/* ボタン */}
      <div className="flex justify-end space-x-3">
        <button
          type="button"
          onClick={onCancel}
          className="px-4 py-2 text-sm font-medium text-gray-700 bg-gray-100 hover:bg-gray-200 rounded-md"
          disabled={isLoading}
        >
          キャンセル
        </button>
        <button
          type="submit"
          disabled={isLoading}
          className="px-4 py-2 text-sm font-medium text-white bg-blue-500 hover:bg-blue-600 rounded-md disabled:opacity-50"
        >
          {isLoading ? "保存中..." : (channel ? "更新" : "追加")}
        </button>
      </div>
    </form>
  );
}
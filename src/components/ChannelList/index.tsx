import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { invoke } from "@tauri-apps/api/core";
import { useState } from "react";
import { useChannelStore } from "../../stores/channelStore";
import { Channel } from "../../types";
import { ChannelForm } from "./ChannelForm";
import { ChannelItem } from "./ChannelItem";

export function ChannelList() {
  const [showAddForm, setShowAddForm] = useState(false);
  const [editingChannel, setEditingChannel] = useState<Channel | null>(null);
  const [filter, setFilter] = useState<'all' | 'twitch' | 'youtube'>('all');

  const { channels, error, fetchChannels } = useChannelStore();
  const queryClient = useQueryClient();

  // チャンネル取得
  const { isLoading } = useQuery({
    queryKey: ["channels"],
    queryFn: async () => {
      await fetchChannels();
      return channels;
    },
    initialData: channels,
  });

  // チャンネル削除ミューテーション
  const deleteMutation = useMutation({
    mutationFn: async (channelId: number) => {
      await invoke("remove_channel", { id: channelId });
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["channels"] });
      queryClient.invalidateQueries({ queryKey: ["live-channels"] });
    },
  });

  // チャンネル有効/無効切り替えミューテーション
  const toggleMutation = useMutation({
    mutationFn: async (channelId: number) => {
      await invoke("toggle_channel", { id: channelId });
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["channels"] });
      queryClient.invalidateQueries({ queryKey: ["live-channels"] });
    },
  });

  const handleDelete = async (channelId: number) => {
    if (window.confirm("このチャンネルを削除しますか？")) {
      try {
        await deleteMutation.mutateAsync(channelId);
      } catch (error) {
        alert("チャンネルの削除に失敗しました: " + String(error));
      }
    }
  };

  const handleToggle = async (channelId: number) => {
    try {
      await toggleMutation.mutateAsync(channelId);
    } catch (error) {
      alert("チャンネルの切り替えに失敗しました: " + String(error));
    }
  };

  // フィルタリングされたチャンネル
  const filteredChannels = channels.filter(channel => {
    if (filter === 'all') return true;
    return channel.platform === filter;
  });

  const handleAddChannel = () => {
    setShowAddForm(true);
    setEditingChannel(null);
  };

  const handleEditChannel = (channel: Channel) => {
    setEditingChannel(channel);
    setShowAddForm(false);
  };

  const handleFormClose = () => {
    setShowAddForm(false);
    setEditingChannel(null);
  };

  const handleFormSuccess = () => {
    setShowAddForm(false);
    setEditingChannel(null);
    queryClient.invalidateQueries({ queryKey: ["channels"] });
  };

  if (isLoading && channels.length === 0) {
    return (
      <div className="flex justify-center items-center h-64">
        <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-500"></div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="p-6">
        <div className="bg-red-50 border border-red-200 rounded-md p-4">
          <div className="text-red-800">
            エラーが発生しました: {error}
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="p-6 space-y-6">
      <div className="flex justify-between items-center">
        <h1 className="text-2xl font-bold text-gray-900">チャンネル管理</h1>
        <button
          onClick={handleAddChannel}
          className="bg-blue-500 hover:bg-blue-600 text-white px-4 py-2 rounded-md font-medium"
        >
          チャンネルを追加
        </button>
      </div>

      {/* フィルター */}
      <div className="flex space-x-2">
        <button
          onClick={() => setFilter('all')}
          className={`px-3 py-1 rounded-md text-sm font-medium ${
            filter === 'all'
              ? 'bg-blue-100 text-blue-800'
              : 'bg-gray-100 text-gray-700 hover:bg-gray-200'
          }`}
        >
          すべて ({channels.length})
        </button>
        <button
          onClick={() => setFilter('twitch')}
          className={`px-3 py-1 rounded-md text-sm font-medium ${
            filter === 'twitch'
              ? 'bg-blue-100 text-blue-800'
              : 'bg-gray-100 text-gray-700 hover:bg-gray-200'
          }`}
        >
          Twitch ({channels.filter(c => c.platform === 'twitch').length})
        </button>
        <button
          onClick={() => setFilter('youtube')}
          className={`px-3 py-1 rounded-md text-sm font-medium ${
            filter === 'youtube'
              ? 'bg-blue-100 text-blue-800'
              : 'bg-gray-100 text-gray-700 hover:bg-gray-200'
          }`}
        >
          YouTube ({channels.filter(c => c.platform === 'youtube').length})
        </button>
      </div>

      {/* フォーム表示 */}
      {(showAddForm || editingChannel) && (
        <div className="bg-white rounded-lg shadow p-6 border border-gray-200">
          <ChannelForm
            channel={editingChannel}
            onSuccess={handleFormSuccess}
            onCancel={handleFormClose}
          />
        </div>
      )}

      {/* チャンネル一覧 */}
      <div className="space-y-4">
        {filteredChannels.length === 0 ? (
          <div className="bg-white rounded-lg shadow p-8 border border-gray-200 text-center">
            <p className="text-gray-500">
              {filter === 'all'
                ? "チャンネルが登録されていません。「チャンネルを追加」ボタンから追加してください。"
                : `${filter === 'twitch' ? 'Twitch' : 'YouTube'} のチャンネルが登録されていません。`
              }
            </p>
          </div>
        ) : (
          filteredChannels.map((channel) => (
            <ChannelItem
              key={`${channel.platform}-${channel.channel_id}`}
              channel={channel}
              onEdit={handleEditChannel}
              onDelete={handleDelete}
              onToggle={handleToggle}
            />
          ))
        )}
      </div>
    </div>
  );
}
import { useQuery } from "@tanstack/react-query";
import { invoke } from "@tauri-apps/api/core";
import { useEffect, useState } from "react";
import { LineChart, Line, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer } from "recharts";
import { ChannelWithStats } from "../../types";

interface StreamStats {
  id?: number;
  stream_id: number;
  collected_at: string;
  viewer_count?: number;
  chat_rate_1min: number;
}

interface LiveChannelCardProps {
  channel: ChannelWithStats;
}

function LiveChannelCard({ channel }: LiveChannelCardProps) {
  return (
    <div className="bg-white rounded-lg shadow p-6 border border-gray-200">
      <div className="flex items-center justify-between">
        <div>
          <h3 className="text-lg font-semibold text-gray-900">{channel.channel_name}</h3>
          <p className="text-sm text-gray-500 capitalize">{channel.platform}</p>
        </div>
        <div className="text-right">
          <div className="text-2xl font-bold text-blue-600">
            {channel.current_viewers || 0}
          </div>
          <div className="text-sm text-gray-500">視聴者</div>
        </div>
      </div>

      {channel.current_title && (
        <div className="mt-4">
          <p className="text-sm text-gray-700 truncate">{channel.current_title}</p>
        </div>
      )}

      <div className="mt-4 flex items-center text-sm text-gray-500">
        <span className="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-green-100 text-green-800">
          ライブ中
        </span>
      </div>
    </div>
  );
}

interface ViewerChartProps {
  data: StreamStats[];
}

function ViewerChart({ data }: ViewerChartProps) {
  // データをグラフ用に変換
  const chartData = data.slice(-20).map(stat => ({
    time: new Date(stat.collected_at).toLocaleTimeString('ja-JP', {
      hour: '2-digit',
      minute: '2-digit'
    }),
    viewers: stat.viewer_count || 0,
    chatRate: stat.chat_rate_1min,
  }));

  return (
    <div className="bg-white rounded-lg shadow p-6 border border-gray-200">
      <h3 className="text-lg font-semibold text-gray-900 mb-4">視聴者数推移</h3>
      <div className="h-64">
        <ResponsiveContainer width="100%" height="100%">
          <LineChart data={chartData}>
            <CartesianGrid strokeDasharray="3 3" />
            <XAxis dataKey="time" />
            <YAxis />
            <Tooltip />
            <Line
              type="monotone"
              dataKey="viewers"
              stroke="#3b82f6"
              strokeWidth={2}
              dot={false}
            />
          </LineChart>
        </ResponsiveContainer>
      </div>
    </div>
  );
}

export function Dashboard() {
  const [statsData, setStatsData] = useState<StreamStats[]>([]);

  // ライブチャンネルを取得
  const { data: liveChannels, isLoading: channelsLoading } = useQuery({
    queryKey: ["live-channels"],
    queryFn: async () => {
      return await invoke<ChannelWithStats[]>("get_live_channels");
    },
    refetchInterval: 30000, // 30秒ごとに更新
  });

  // 最新の統計データを取得
  const { data: recentStats } = useQuery({
    queryKey: ["recent-stats"],
    queryFn: async () => {
      return await invoke<StreamStats[]>("get_stream_stats", {
        query: {
          start_time: new Date(Date.now() - 3600000).toISOString(), // 1時間前から
        },
      });
    },
    refetchInterval: 10000, // 10秒ごとに更新
  });

  useEffect(() => {
    if (recentStats) {
      setStatsData(recentStats);
    }
  }, [recentStats]);

  const totalViewers = liveChannels?.reduce((sum, channel) => sum + (channel.current_viewers || 0), 0) || 0;

  return (
    <div className="p-6 space-y-6">
      <div className="flex justify-between items-center">
        <h1 className="text-2xl font-bold text-gray-900">ダッシュボード</h1>
        <div className="text-sm text-gray-500">
          最終更新: {new Date().toLocaleTimeString('ja-JP')}
        </div>
      </div>

      {/* 概要統計 */}
      <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
        <div className="bg-white rounded-lg shadow p-6 border border-gray-200">
          <div className="flex items-center">
            <div className="flex-shrink-0">
              <div className="w-8 h-8 bg-blue-500 rounded-full flex items-center justify-center">
                <span className="text-white text-sm font-bold">L</span>
              </div>
            </div>
            <div className="ml-4">
              <h3 className="text-lg font-semibold text-gray-900">{liveChannels?.length || 0}</h3>
              <p className="text-sm text-gray-500">ライブ中チャンネル</p>
            </div>
          </div>
        </div>

        <div className="bg-white rounded-lg shadow p-6 border border-gray-200">
          <div className="flex items-center">
            <div className="flex-shrink-0">
              <div className="w-8 h-8 bg-green-500 rounded-full flex items-center justify-center">
                <span className="text-white text-sm font-bold">👥</span>
              </div>
            </div>
            <div className="ml-4">
              <h3 className="text-lg font-semibold text-gray-900">{totalViewers.toLocaleString()}</h3>
              <p className="text-sm text-gray-500">総視聴者数</p>
            </div>
          </div>
        </div>

        <div className="bg-white rounded-lg shadow p-6 border border-gray-200">
          <div className="flex items-center">
            <div className="flex-shrink-0">
              <div className="w-8 h-8 bg-purple-500 rounded-full flex items-center justify-center">
                <span className="text-white text-sm font-bold">💬</span>
              </div>
            </div>
            <div className="ml-4">
              <h3 className="text-lg font-semibold text-gray-900">
                {statsData.length > 0 ? statsData[statsData.length - 1]?.chat_rate_1min || 0 : 0}
              </h3>
              <p className="text-sm text-gray-500">1分間チャット数</p>
            </div>
          </div>
        </div>
      </div>

      {/* チャートとライブチャンネル */}
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        <ViewerChart data={statsData} />

        <div className="bg-white rounded-lg shadow p-6 border border-gray-200">
          <h3 className="text-lg font-semibold text-gray-900 mb-4">ライブ中チャンネル</h3>
          <div className="space-y-4 max-h-96 overflow-y-auto">
            {channelsLoading ? (
              <div className="text-center py-8">
                <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-500 mx-auto"></div>
                <p className="text-sm text-gray-500 mt-2">読み込み中...</p>
              </div>
            ) : liveChannels && liveChannels.length > 0 ? (
              liveChannels.map((channel) => (
                <LiveChannelCard key={`${channel.platform}-${channel.channel_id}`} channel={channel} />
              ))
            ) : (
              <div className="text-center py-8 text-gray-500">
                現在ライブ中のチャンネルはありません
              </div>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}
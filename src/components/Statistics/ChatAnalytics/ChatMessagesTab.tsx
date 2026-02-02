import { useState } from 'react';
import { useQuery } from '@tanstack/react-query';
import { invoke } from '@tauri-apps/api/core';
import type { ChatMessage, StreamInfo } from '../../../types';
import { LoadingSpinner } from '../../common/LoadingSpinner';

interface ChatMessagesTabProps {
  channelId: number | null;
  startTime: string;
  endTime: string;
}

const ChatMessagesTab = ({ channelId, startTime, endTime }: ChatMessagesTabProps) => {
  const [selectedStreamId, setSelectedStreamId] = useState<number | null>(null);
  const [currentPage, setCurrentPage] = useState(1);
  const messagesPerPage = 100;

  // Get streams for the selected channel
  const { data: streams } = useQuery({
    queryKey: ['channelStreams', channelId],
    queryFn: async () => {
      if (!channelId) return [];
      const result = await invoke<StreamInfo[]>('get_channel_streams', {
        channelId,
        limit: 50,
      });
      return result;
    },
    enabled: !!channelId,
  });

  // Get chat messages
  const { data: messages, isLoading } = useQuery({
    queryKey: [
      'chatMessages',
      channelId,
      selectedStreamId,
      startTime,
      endTime,
      currentPage,
    ],
    queryFn: async () => {
      const result = await invoke<ChatMessage[]>('get_chat_messages', {
        channelId,
        streamId: selectedStreamId,
        startTime,
        endTime,
        limit: messagesPerPage,
        offset: (currentPage - 1) * messagesPerPage,
      });
      return result;
    },
  });

  const totalPages = messages && messages.length === messagesPerPage ? currentPage + 1 : currentPage;

  const getBadgeColor = (badge: string): string => {
    if (badge.includes('broadcaster')) return 'bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-200';
    if (badge.includes('moderator')) return 'bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200';
    if (badge.includes('vip')) return 'bg-purple-100 text-purple-800 dark:bg-purple-900 dark:text-purple-200';
    if (badge.includes('subscriber')) return 'bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-200';
    return 'bg-gray-100 text-gray-800 dark:bg-gray-700 dark:text-gray-300';
  };

  return (
    <div className="space-y-6">
      {/* Filters */}
      <div className="bg-white dark:bg-gray-800 rounded-lg p-4 shadow">
        <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
          {/* Stream Filter */}
          <div>
            <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
              配信
            </label>
            <select
              value={selectedStreamId || ''}
              onChange={(e) => {
                setSelectedStreamId(e.target.value ? Number(e.target.value) : null);
                setCurrentPage(1);
              }}
              className="w-full px-3 py-2 bg-white dark:bg-gray-700 border border-gray-300 dark:border-gray-600 rounded-md text-gray-900 dark:text-white"
            >
              <option value="">全ての配信</option>
              {streams?.map((stream) => (
                <option key={stream.id} value={stream.id}>
                  {new Date(stream.started_at).toLocaleDateString('ja-JP')} - {stream.title}
                </option>
              ))}
            </select>
          </div>

          {/* Summary */}
          <div className="flex items-center">
            <div className="text-sm text-gray-600 dark:text-gray-400">
              {messages && messages.length > 0 ? (
                <>
                  {(currentPage - 1) * messagesPerPage + 1} - {(currentPage - 1) * messagesPerPage + messages.length} 件
                  のメッセージを表示中
                </>
              ) : (
                'メッセージがありません'
              )}
            </div>
          </div>
        </div>
      </div>

      {/* Messages Table */}
      <div className="bg-white dark:bg-gray-800 rounded-lg p-6 shadow">
        <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-4">
          チャットメッセージ
        </h3>

        {isLoading ? (
          <LoadingSpinner />
        ) : messages && messages.length > 0 ? (
          <>
            <div className="overflow-x-auto">
              <table className="min-w-full divide-y divide-gray-200 dark:divide-gray-700">
                <thead className="bg-gray-50 dark:bg-gray-900">
                  <tr>
                    <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">
                      時刻
                    </th>
                    <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">
                      ユーザー
                    </th>
                    <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">
                      バッジ
                    </th>
                    <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">
                      メッセージ
                    </th>
                  </tr>
                </thead>
                <tbody className="bg-white dark:bg-gray-800 divide-y divide-gray-200 dark:divide-gray-700">
                  {messages.map((message, index) => (
                    <tr key={message.id || index}>
                      <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500 dark:text-gray-400">
                        {new Date(message.timestamp).toLocaleString('ja-JP', {
                          month: '2-digit',
                          day: '2-digit',
                          hour: '2-digit',
                          minute: '2-digit',
                          second: '2-digit',
                        })}
                      </td>
                      <td className="px-6 py-4 whitespace-nowrap text-sm font-medium text-gray-900 dark:text-white">
                        {message.user_name}
                      </td>
                      <td className="px-6 py-4 whitespace-nowrap text-sm">
                        <div className="flex flex-wrap gap-1">
                          {message.badges && message.badges.length > 0 ? (
                            message.badges.map((badge, idx) => (
                              <span
                                key={idx}
                                className={`inline-flex items-center px-2 py-0.5 rounded text-xs font-medium ${getBadgeColor(
                                  badge
                                )}`}
                              >
                                {badge}
                              </span>
                            ))
                          ) : (
                            <span className="text-gray-400 dark:text-gray-500 text-xs">-</span>
                          )}
                        </div>
                      </td>
                      <td className="px-6 py-4 text-sm text-gray-900 dark:text-white">
                        <div className="max-w-2xl break-words">{message.message}</div>
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>

            {/* Pagination */}
            <div className="mt-4 flex items-center justify-between">
              <button
                onClick={() => setCurrentPage((p) => Math.max(1, p - 1))}
                disabled={currentPage === 1}
                className="px-4 py-2 bg-white dark:bg-gray-700 border border-gray-300 dark:border-gray-600 rounded-md text-sm font-medium text-gray-700 dark:text-gray-300 hover:bg-gray-50 dark:hover:bg-gray-600 disabled:opacity-50 disabled:cursor-not-allowed"
              >
                前へ
              </button>
              <span className="text-sm text-gray-700 dark:text-gray-300">
                ページ {currentPage}
              </span>
              <button
                onClick={() => setCurrentPage((p) => p + 1)}
                disabled={!messages || messages.length < messagesPerPage}
                className="px-4 py-2 bg-white dark:bg-gray-700 border border-gray-300 dark:border-gray-600 rounded-md text-sm font-medium text-gray-700 dark:text-gray-300 hover:bg-gray-50 dark:hover:bg-gray-600 disabled:opacity-50 disabled:cursor-not-allowed"
              >
                次へ
              </button>
            </div>
          </>
        ) : (
          <p className="text-gray-500 dark:text-gray-400 text-center py-8">
            データがありません
          </p>
        )}
      </div>
    </div>
  );
};

export default ChatMessagesTab;

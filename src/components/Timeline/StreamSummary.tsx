import React from 'react';
import { StreamInfo } from '../../types';

interface StreamSummaryProps {
  streamInfo: StreamInfo;
}

const StreamSummary: React.FC<StreamSummaryProps> = ({ streamInfo }) => {
  const formatDuration = (minutes: number): string => {
    const hours = Math.floor(minutes / 60);
    const mins = Math.floor(minutes % 60);
    return `${hours}時間${mins}分`;
  };

  const formatDate = (dateStr: string): string => {
    const date = new Date(dateStr);
    return date.toLocaleString('ja-JP', {
      year: 'numeric',
      month: '2-digit',
      day: '2-digit',
      hour: '2-digit',
      minute: '2-digit',
    });
  };

  return (
    <div className="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 p-6">
      <div className="flex items-center justify-between mb-4">
        <h2 className="text-xl font-bold text-gray-900 dark:text-white">
          {streamInfo.title || '(タイトルなし)'}
        </h2>
        {streamInfo.ended_at ? (
          <span className="px-3 py-1 text-sm font-medium bg-gray-100 dark:bg-gray-700 text-gray-700 dark:text-gray-300 rounded">
            終了
          </span>
        ) : (
          <span className="px-3 py-1 text-sm font-medium bg-red-100 dark:bg-red-900/30 text-red-700 dark:text-red-400 rounded">
            配信中
          </span>
        )}
      </div>

      <div className="text-gray-600 dark:text-gray-400 mb-6">
        <p className="mb-1">配信者: <span className="font-medium text-gray-900 dark:text-white">{streamInfo.channel_name}</span></p>
        <p className="mb-1">カテゴリ: <span className="font-medium text-gray-900 dark:text-white">{streamInfo.category || '(カテゴリなし)'}</span></p>
        <p className="mb-1">開始: <span className="font-medium text-gray-900 dark:text-white">{formatDate(streamInfo.started_at)}</span></p>
        {streamInfo.ended_at && (
          <p>終了: <span className="font-medium text-gray-900 dark:text-white">{formatDate(streamInfo.ended_at)}</span></p>
        )}
      </div>

      <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
        <div className="bg-blue-50 dark:bg-blue-900/20 rounded-lg p-4">
          <p className="text-sm text-blue-600 dark:text-blue-400 mb-1">配信時間</p>
          <p className="text-2xl font-bold text-blue-900 dark:text-blue-300">
            {formatDuration(streamInfo.duration_minutes)}
          </p>
        </div>

        <div className="bg-green-50 dark:bg-green-900/20 rounded-lg p-4">
          <p className="text-sm text-green-600 dark:text-green-400 mb-1">ピーク視聴者数</p>
          <p className="text-2xl font-bold text-green-900 dark:text-green-300">
            {streamInfo.peak_viewers.toLocaleString()}
          </p>
        </div>

        <div className="bg-purple-50 dark:bg-purple-900/20 rounded-lg p-4">
          <p className="text-sm text-purple-600 dark:text-purple-400 mb-1">平均視聴者数</p>
          <p className="text-2xl font-bold text-purple-900 dark:text-purple-300">
            {streamInfo.avg_viewers.toLocaleString()}
          </p>
        </div>

        <div className="bg-orange-50 dark:bg-orange-900/20 rounded-lg p-4">
          <p className="text-sm text-orange-600 dark:text-orange-400 mb-1">ピーク/平均比</p>
          <p className="text-2xl font-bold text-orange-900 dark:text-orange-300">
            {streamInfo.avg_viewers > 0 
              ? (streamInfo.peak_viewers / streamInfo.avg_viewers).toFixed(2) 
              : 'N/A'}
          </p>
        </div>
      </div>
    </div>
  );
};

export default StreamSummary;

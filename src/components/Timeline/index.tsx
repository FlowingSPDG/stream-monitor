import React, { useState } from 'react';
import { ErrorBoundary } from '../common/ErrorBoundary';
import StreamSelector from './StreamSelector';
import TimelineChart from './TimelineChart';
import StreamSummary from './StreamSummary';
import { StreamTimelineData } from '../../types';

const Timeline: React.FC = () => {
  const [selectedTimeline, setSelectedTimeline] = useState<StreamTimelineData | null>(null);

  return (
    <ErrorBoundary>
      <div className="p-6 space-y-6">
        <div className="flex items-center justify-between mb-6">
          <h1 className="text-2xl font-bold text-gray-900 dark:text-white">Timeline</h1>
        </div>

        <StreamSelector onTimelineSelect={setSelectedTimeline} />

        {selectedTimeline && (
          <div className="space-y-6">
            <StreamSummary streamInfo={selectedTimeline.stream_info} />
            <TimelineChart timelineData={selectedTimeline} />
          </div>
        )}

        {!selectedTimeline && (
          <div className="text-center py-12 text-gray-500 dark:text-gray-400">
            配信者とストリームを選択して、タイムライン表示を開始してください
          </div>
        )}
      </div>
    </ErrorBoundary>
  );
};

export default Timeline;

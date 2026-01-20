export interface Channel {
  id?: number;
  platform: 'twitch' | 'youtube';
  channel_id: string;
  channel_name: string;
  enabled: boolean;
  poll_interval: number;
  created_at?: string;
  updated_at?: string;
}

export interface StreamStats {
  id?: number;
  stream_id: number;
  collected_at: string;
  viewer_count?: number;
  chat_rate_1min: number;
}

export interface ChannelWithStats extends Channel {
  is_live: boolean;
  current_viewers?: number;
  current_title?: string;
}

export interface StreamStatsQuery {
  stream_id?: number;
  channel_id?: number;
  start_time?: string;
  end_time?: string;
}

export interface ExportQuery {
  channel_id?: number;
  start_time?: string;
  end_time?: string;
}

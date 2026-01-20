import { useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { useConfigStore } from '../../stores/configStore';

interface OAuthLoginProps {
  platform: 'twitch' | 'youtube';
  clientId: string;
  clientSecret: string;
}

export function OAuthLogin({ platform, clientId, clientSecret }: OAuthLoginProps) {
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const { checkTokens } = useConfigStore();

  const handleLogin = async () => {
    if (!clientId || !clientSecret) {
      setError('Client ID と Client Secret を設定してください');
      return;
    }

    setLoading(true);
    setError(null);

    try {
      const command = platform === 'twitch' ? 'login_with_twitch' : 'login_with_youtube';
      await invoke<string>(command, {
        config: {
          client_id: clientId,
          client_secret: clientSecret,
        },
        port: platform === 'twitch' ? 8080 : 8081,
      });

      // トークンの存在を確認
      await checkTokens();
      setError(null);
    } catch (err) {
      setError(String(err));
    } finally {
      setLoading(false);
    }
  };

  const platformName = platform === 'twitch' ? 'Twitch' : 'YouTube';

  return (
    <div className="space-y-4">
      <button
        onClick={handleLogin}
        disabled={loading || !clientId || !clientSecret}
        className="px-4 py-2 bg-purple-600 hover:bg-purple-700 disabled:bg-gray-400 text-white rounded-lg transition-colors"
      >
        {loading ? `${platformName}に接続中...` : `${platformName}でログイン`}
      </button>
      {error && (
        <div className="text-red-500 text-sm">{error}</div>
      )}
    </div>
  );
}

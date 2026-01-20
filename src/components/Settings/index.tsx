import { useState, useEffect } from 'react';
import { OAuthLogin } from './OAuthLogin';
import { useConfigStore } from '../../stores/configStore';

export function Settings() {
  const [twitchClientId, setTwitchClientId] = useState('');
  const [twitchClientSecret, setTwitchClientSecret] = useState('');
  const [youtubeClientId, setYoutubeClientId] = useState('');
  const [youtubeClientSecret, setYoutubeClientSecret] = useState('');
  
  const { hasTwitchToken, hasYouTubeToken, checkTokens } = useConfigStore();

  useEffect(() => {
    checkTokens();
  }, [checkTokens]);

  return (
    <div className="p-6 space-y-8">
      <h1 className="text-2xl font-bold">設定</h1>

      <section className="space-y-4">
        <h2 className="text-xl font-semibold">Twitch API設定</h2>
        <div className="space-y-2">
          <label className="block text-sm font-medium">
            Client ID
            <input
              type="text"
              value={twitchClientId}
              onChange={(e) => setTwitchClientId(e.target.value)}
              className="mt-1 block w-full px-3 py-2 border border-gray-300 rounded-md"
              placeholder="Twitch Client IDを入力"
            />
          </label>
          <label className="block text-sm font-medium">
            Client Secret
            <input
              type="password"
              value={twitchClientSecret}
              onChange={(e) => setTwitchClientSecret(e.target.value)}
              className="mt-1 block w-full px-3 py-2 border border-gray-300 rounded-md"
              placeholder="Twitch Client Secretを入力"
            />
          </label>
          {hasTwitchToken && (
            <div className="text-green-600 text-sm">✓ Twitchに接続済み</div>
          )}
          <OAuthLogin
            platform="twitch"
            clientId={twitchClientId}
            clientSecret={twitchClientSecret}
          />
        </div>
      </section>

      <section className="space-y-4">
        <h2 className="text-xl font-semibold">YouTube API設定</h2>
        <div className="space-y-2">
          <label className="block text-sm font-medium">
            Client ID
            <input
              type="text"
              value={youtubeClientId}
              onChange={(e) => setYoutubeClientId(e.target.value)}
              className="mt-1 block w-full px-3 py-2 border border-gray-300 rounded-md"
              placeholder="YouTube Client IDを入力"
            />
          </label>
          <label className="block text-sm font-medium">
            Client Secret
            <input
              type="password"
              value={youtubeClientSecret}
              onChange={(e) => setYoutubeClientSecret(e.target.value)}
              className="mt-1 block w-full px-3 py-2 border border-gray-300 rounded-md"
              placeholder="YouTube Client Secretを入力"
            />
          </label>
          {hasYouTubeToken && (
            <div className="text-green-600 text-sm">✓ YouTubeに接続済み</div>
          )}
          <OAuthLogin
            platform="youtube"
            clientId={youtubeClientId}
            clientSecret={youtubeClientSecret}
          />
        </div>
      </section>
    </div>
  );
}

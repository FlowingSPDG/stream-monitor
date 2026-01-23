import { useState, useEffect } from 'react';
import { useConfigStore } from '../../stores/configStore';

interface OAuthConfigFormProps {
  platform: 'twitch' | 'youtube';
  onClose?: () => void;
}

export function OAuthConfigForm({ platform, onClose }: OAuthConfigFormProps) {
  const [clientId, setClientId] = useState('');
  const [clientSecret, setClientSecret] = useState('');
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState<string | null>(null);

  const { getOAuthConfig, saveOAuthConfig, deleteOAuthConfig } = useConfigStore();

  const platformName = platform === 'twitch' ? 'Twitch' : 'YouTube';

  // コンポーネントマウント時に既存設定を読み込み
  useEffect(() => {
    const loadExistingConfig = async () => {
      try {
        const config = await getOAuthConfig(platform);
        if (config.client_id) {
          setClientId(config.client_id);
        }
        if (config.client_secret) {
          setClientSecret(config.client_secret);
        }
      } catch (err) {
        // 設定が存在しない場合は何もしない（新規設定時）
        console.log(`${platformName} OAuth config not found, starting fresh`);
      }
    };

    loadExistingConfig();
  }, [platform, getOAuthConfig, platformName]);

  const handleSave = async () => {
    // Client IDは必須
    if (!clientId.trim()) {
      setError('Client ID は必須です');
      return;
    }

    // Twitchの場合、Device Code FlowではClient Secretは不要
    // YouTubeの場合、Client Secretは必須
    if (platform === 'youtube' && !clientSecret.trim()) {
      setError('YouTube OAuth では Client ID と Client Secret の両方が必要です');
      return;
    }

    setLoading(true);
    setError(null);
    setSuccess(null);

    try {
      // Client Secretが空でない場合のみ送信
      const secret = clientSecret.trim() || undefined;
      await saveOAuthConfig(platform, clientId.trim(), secret);
      setSuccess(`${platformName} OAuth設定を保存しました`);
      if (onClose) {
        setTimeout(() => onClose(), 2000); // 2秒後に閉じる
      }
    } catch (err) {
      const errorMessage = String(err);
      setError(errorMessage);
    } finally {
      setLoading(false);
    }
  };

  const handleDelete = async () => {
    if (!confirm(`${platformName}のOAuth設定を削除しますか？\n\n注意: この操作により、保存されたClient IDとClient Secretが完全に削除されます。`)) {
      return;
    }

    setLoading(true);
    setError(null);
    setSuccess(null);

    try {
      await deleteOAuthConfig(platform);
      setSuccess(`${platformName} OAuth設定を削除しました`);
      setClientId('');
      setClientSecret('');
      if (onClose) {
        setTimeout(() => onClose(), 2000); // 2秒後に閉じる
      }
    } catch (err) {
      const errorMessage = String(err);
      setError(errorMessage);
    } finally {
      setLoading(false);
    }
  };

  const handleClose = () => {
    setClientId('');
    setClientSecret('');
    setError(null);
    setSuccess(null);
    if (onClose) {
      onClose();
    }
  };

  return (
    <div className="space-y-4 p-4 border border-gray-200 dark:border-slate-600 rounded-lg bg-gray-50 dark:bg-slate-800">
      <div className="flex items-center justify-between">
        <h3 className="text-lg font-semibold text-gray-900 dark:text-gray-100">
          {platformName} OAuth設定
        </h3>
        {onClose && (
          <button
            onClick={handleClose}
            className="text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200"
          >
            ✕
          </button>
        )}
      </div>

      <div className="text-sm text-gray-600 dark:text-gray-400">
        {platform === 'twitch' ? (
          <>
            Twitch APIを使用するには、Twitch Developer ConsoleでOAuthアプリケーションを作成し、Client IDを取得してください。
            <br />
            <span className="text-xs">
              💡 Device Code Flowを使用するため、Client Secretは不要です（オプション）。
            </span>
          </>
        ) : (
          `${platformName} APIを使用するには、${platformName} Developer ConsoleでOAuthアプリケーションを作成し、Client IDとClient Secretを取得してください。`
        )}
      </div>

      <div className="space-y-4">
        <label className="block text-sm font-medium text-gray-700 dark:text-gray-300">
          Client ID
          <input
            type="text"
            value={clientId}
            onChange={(e) => setClientId(e.target.value)}
            className="input-field mt-1"
            placeholder={`${platformName} Client IDを入力`}
            disabled={loading}
          />
        </label>
        <label className="block text-sm font-medium text-gray-700 dark:text-gray-300">
          Client Secret {platform === 'twitch' && <span className="text-xs text-gray-500">(オプション)</span>}
          <input
            type="password"
            value={clientSecret}
            onChange={(e) => setClientSecret(e.target.value)}
            className="input-field mt-1"
            placeholder={platform === 'twitch' ? 'Twitch Client Secret (不要)' : `${platformName} Client Secretを入力`}
            disabled={loading}
          />
        </label>

        <div className="flex space-x-3">
          <button
            onClick={handleSave}
            disabled={loading || !clientId.trim() || (platform === 'youtube' && !clientSecret.trim())}
            className="px-4 py-2 bg-green-600 hover:bg-green-700 disabled:bg-gray-400 text-white rounded-lg transition-colors text-sm font-medium"
          >
            {loading ? '保存中...' : '保存'}
          </button>
          <button
            onClick={handleDelete}
            disabled={loading}
            className="px-4 py-2 bg-red-600 hover:bg-red-700 disabled:bg-gray-400 text-white rounded-lg transition-colors text-sm font-medium"
          >
            {loading ? '削除中...' : '削除'}
          </button>
        </div>

        {error && (
          <div className="text-red-500 text-sm p-2 bg-red-50 dark:bg-red-900/20 rounded">
            {error}
          </div>
        )}

        {success && (
          <div className="text-green-600 text-sm p-2 bg-green-50 dark:bg-green-900/20 rounded">
            {success}
          </div>
        )}
      </div>
    </div>
  );
}
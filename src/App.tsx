import { useState } from "react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { Settings } from "./components/Settings";
import { Dashboard } from "./components/Dashboard";
import { ChannelList } from "./components/ChannelList";
import { Statistics } from "./components/Statistics";
import { Export } from "./components/Export";
import { ErrorBoundary } from "./components/common/ErrorBoundary";
import "./App.css";

const queryClient = new QueryClient();

type Tab = "dashboard" | "channels" | "statistics" | "export" | "settings";

function App() {
  const [activeTab, setActiveTab] = useState<Tab>("dashboard");

  const tabs: { id: Tab; label: string; component: React.ReactNode }[] = [
    {
      id: "dashboard",
      label: "ダッシュボード",
      component: <Dashboard />
    },
    {
      id: "channels",
      label: "チャンネル管理",
      component: <ChannelList />
    },
    {
      id: "statistics",
      label: "統計閲覧",
      component: <Statistics />
    },
    {
      id: "export",
      label: "エクスポート",
      component: <Export />
    },
    {
      id: "settings",
      label: "設定",
      component: <Settings />
    },
  ];

  return (
    <ErrorBoundary>
      <QueryClientProvider client={queryClient}>
        <div className="min-h-screen bg-gray-50">
          {/* ナビゲーションバー */}
          <nav className="bg-white shadow-sm border-b">
            <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
              <div className="flex justify-between h-16">
                <div className="flex">
                  <div className="flex-shrink-0 flex items-center">
                    <h1 className="text-xl font-bold text-gray-900">Stream Stats Collector</h1>
                  </div>
                </div>
              </div>
            </div>

            {/* タブナビゲーション */}
            <div className="border-t border-gray-200">
              <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
                <nav className="flex space-x-8">
                  {tabs.map((tab) => (
                    <button
                      key={tab.id}
                      onClick={() => setActiveTab(tab.id)}
                      className={`py-4 px-1 border-b-2 font-medium text-sm ${
                        activeTab === tab.id
                          ? "border-blue-500 text-blue-600"
                          : "border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300"
                      }`}
                    >
                      {tab.label}
                    </button>
                  ))}
                </nav>
              </div>
            </div>
          </nav>

          {/* メインコンテンツ */}
          <main className="flex-1">
            {tabs.find(tab => tab.id === activeTab)?.component}
          </main>
        </div>
      </QueryClientProvider>
    </ErrorBoundary>
  );
}

export default App;

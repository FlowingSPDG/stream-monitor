import { BarChart as RechartsBarChart, Bar, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer, Legend } from "recharts";
import { Tooltip as InfoTooltip } from "../Tooltip";

interface ChartDataPoint {
  [key: string]: string | number | undefined;
}

interface BarChartProps {
  data: ChartDataPoint[];
  dataKey: string;
  xAxisKey?: string;
  color?: string;
  title?: string;
  tooltipDescription?: string;
  height?: number;
  showLegend?: boolean;
  yAxisLabel?: string;
}

export function BarChart({
  data,
  dataKey,
  xAxisKey = "name",
  color = "#10b981",
  title,
  tooltipDescription,
  height = 300,
  showLegend = false,
  yAxisLabel,
}: BarChartProps) {
  return (
    <div className="w-full">
      {title && (
        <div className="flex items-center gap-2 mb-4">
          <h3 className="text-lg font-semibold text-gray-900 dark:text-gray-100">{title}</h3>
          {tooltipDescription && (
            <InfoTooltip content={tooltipDescription}>
              <span className="text-xs text-gray-400 cursor-help">ℹ️</span>
            </InfoTooltip>
          )}
        </div>
      )}
      <div style={{ height: `${height}px` }}>
        <ResponsiveContainer width="100%" height="100%">
          <RechartsBarChart data={data} margin={{ top: 5, right: 30, left: 20, bottom: 5 }}>
            <CartesianGrid strokeDasharray="3 3" stroke="#f0f0f0" />
            <XAxis
              dataKey={xAxisKey}
              tick={{ fontSize: 12 }}
              tickLine={{ stroke: '#e0e0e0' }}
            />
            <YAxis
              tick={{ fontSize: 12 }}
              tickLine={{ stroke: '#e0e0e0' }}
              label={yAxisLabel ? { value: yAxisLabel, angle: -90, position: 'insideLeft' } : undefined}
            />
            <Tooltip
              contentStyle={{
                backgroundColor: '#ffffff',
                border: '1px solid #e0e0e0',
                borderRadius: '6px',
                fontSize: '12px'
              }}
            />
            {showLegend && <Legend />}
            <Bar
              dataKey={dataKey}
              fill={color}
              radius={[4, 4, 0, 0]}
            />
          </RechartsBarChart>
        </ResponsiveContainer>
      </div>
    </div>
  );
}
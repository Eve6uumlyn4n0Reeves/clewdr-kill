import React from 'react';

/**
 * 主题配置测试组件
 * 用于验证 Tailwind 配置和赛博朋克主题是否正确加载
 */
const ThemeTest: React.FC = () => {
  return (
    <div className="min-h-screen bg-background p-8 space-y-8">
      {/* 标题区域 */}
      <div className="text-center space-y-4">
        <h1 className="text-4xl font-bold text-gradient">
          ClewdR Kill Edition
        </h1>
        <p className="text-muted text-lg">
          赛博朋克主题配置测试
        </p>
      </div>

      {/* 色彩测试 */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
        {/* 主色调测试 */}
        <div className="card space-y-4">
          <h2 className="text-xl font-semibold text-foreground">主色调</h2>
          <div className="space-y-2">
            <div className="w-full h-12 bg-primary rounded-lg glow-primary flex items-center justify-center">
              <span className="text-white font-medium">Primary</span>
            </div>
            <div className="w-full h-8 bg-primary-600 rounded flex items-center justify-center">
              <span className="text-white text-sm">Primary 600</span>
            </div>
          </div>
        </div>

        {/* 状态色测试 */}
        <div className="card space-y-4">
          <h2 className="text-xl font-semibold text-foreground">状态色</h2>
          <div className="space-y-2">
            <div className="w-full h-8 bg-success rounded glow-success flex items-center justify-center">
              <span className="text-white text-sm">Success</span>
            </div>
            <div className="w-full h-8 bg-danger rounded glow-danger flex items-center justify-center">
              <span className="text-white text-sm">Danger</span>
            </div>
            <div className="w-full h-8 bg-warning rounded glow-warning flex items-center justify-center">
              <span className="text-white text-sm">Warning</span>
            </div>
            <div className="w-full h-8 bg-info rounded glow-info flex items-center justify-center">
              <span className="text-white text-sm">Info</span>
            </div>
          </div>
        </div>

        {/* 背景色测试 */}
        <div className="card space-y-4">
          <h2 className="text-xl font-semibold text-foreground">背景色</h2>
          <div className="space-y-2">
            <div className="w-full h-8 bg-surface rounded border border-border flex items-center justify-center">
              <span className="text-foreground text-sm">Surface</span>
            </div>
            <div className="w-full h-8 bg-surfaceHighlight rounded border border-borderHighlight flex items-center justify-center">
              <span className="text-foreground text-sm">Surface Highlight</span>
            </div>
            <div className="w-full h-8 bg-surfaceHover rounded flex items-center justify-center">
              <span className="text-foreground text-sm">Surface Hover</span>
            </div>
          </div>
        </div>
      </div>

      {/* 按钮测试 */}
      <div className="card space-y-4">
        <h2 className="text-xl font-semibold text-foreground">按钮样式</h2>
        <div className="flex flex-wrap gap-4">
          <button className="btn-primary">Primary Button</button>
          <button className="btn-secondary">Secondary Button</button>
          <button className="btn-ghost">Ghost Button</button>
          <button className="btn-danger">Danger Button</button>
        </div>
      </div>

      {/* 状态徽章测试 */}
      <div className="card space-y-4">
        <h2 className="text-xl font-semibold text-foreground">状态徽章</h2>
        <div className="flex flex-wrap gap-4">
          <span className="badge-pending">Pending</span>
          <span className="badge-checking">Checking</span>
          <span className="badge-banned">Banned</span>
          <span className="badge-alive">Alive</span>
        </div>
      </div>

      {/* 输入框测试 */}
      <div className="card space-y-4">
        <h2 className="text-xl font-semibold text-foreground">输入框</h2>
        <div className="space-y-4">
          <input
            type="text"
            placeholder="普通输入框"
            className="input"
          />
          <textarea
            placeholder="文本区域"
            className="input h-24 resize-none"
          />
        </div>
      </div>

      {/* 脉冲点测试 */}
      <div className="card space-y-4">
        <h2 className="text-xl font-semibold text-foreground">脉冲指示器</h2>
        <div className="flex items-center gap-6">
          <div className="flex items-center gap-2">
            <div className="pulse-dot-success"></div>
            <span className="text-sm text-muted">Success</span>
          </div>
          <div className="flex items-center gap-2">
            <div className="pulse-dot-danger"></div>
            <span className="text-sm text-muted">Danger</span>
          </div>
          <div className="flex items-center gap-2">
            <div className="pulse-dot-warning"></div>
            <span className="text-sm text-muted">Warning</span>
          </div>
          <div className="flex items-center gap-2">
            <div className="pulse-dot-info"></div>
            <span className="text-sm text-muted">Info</span>
          </div>
        </div>
      </div>

      {/* 玻璃拟态效果测试 */}
      <div className="space-y-4">
        <h2 className="text-xl font-semibold text-foreground">玻璃拟态效果</h2>
        <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
          <div className="glass p-6 rounded-xl">
            <h3 className="text-lg font-medium text-foreground mb-2">Glass Effect</h3>
            <p className="text-muted text-sm">
              这是标准的玻璃拟态效果，具有半透明背景和模糊效果。
            </p>
          </div>
          <div className="glass-strong p-6 rounded-xl">
            <h3 className="text-lg font-medium text-foreground mb-2">Glass Strong</h3>
            <p className="text-muted text-sm">
              这是增强版的玻璃拟态效果，具有更强的模糊和更高的不透明度。
            </p>
          </div>
        </div>
      </div>

      {/* 动画测试 */}
      <div className="card space-y-4">
        <h2 className="text-xl font-semibold text-foreground">动画效果</h2>
        <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
          <div className="p-4 bg-surfaceHighlight rounded-lg animate-fade-in">
            <span className="text-sm text-muted">Fade In</span>
          </div>
          <div className="p-4 bg-surfaceHighlight rounded-lg animate-slide-up">
            <span className="text-sm text-muted">Slide Up</span>
          </div>
          <div className="p-4 bg-surfaceHighlight rounded-lg animate-scale-in">
            <span className="text-sm text-muted">Scale In</span>
          </div>
          <div className="p-4 bg-surfaceHighlight rounded-lg animate-pulse-slow">
            <span className="text-sm text-muted">Pulse Slow</span>
          </div>
        </div>
      </div>

      {/* 数据展示测试 */}
      <div className="card space-y-4">
        <h2 className="text-xl font-semibold text-foreground">数据展示</h2>
        <div className="grid grid-cols-2 md:grid-cols-4 gap-6">
          <div className="text-center space-y-2">
            <div className="data-value text-primary">1,234</div>
            <div className="data-label">Total Requests</div>
          </div>
          <div className="text-center space-y-2">
            <div className="data-value text-success">856</div>
            <div className="data-label">Success Count</div>
          </div>
          <div className="text-center space-y-2">
            <div className="data-value text-danger">378</div>
            <div className="data-label">Banned Count</div>
          </div>
          <div className="text-center space-y-2">
            <div className="data-value text-warning">42</div>
            <div className="data-label">Pending Count</div>
          </div>
        </div>
      </div>

      {/* 进度条测试 */}
      <div className="card space-y-4">
        <h2 className="text-xl font-semibold text-foreground">进度条</h2>
        <div className="space-y-4">
          <div>
            <div className="flex justify-between text-sm text-muted mb-2">
              <span>处理进度</span>
              <span>75%</span>
            </div>
            <div className="progress">
              <div className="progress-bar" style={{ width: '75%' }}></div>
            </div>
          </div>
          <div>
            <div className="flex justify-between text-sm text-muted mb-2">
              <span>成功率</span>
              <span>92%</span>
            </div>
            <div className="progress">
              <div className="progress-bar bg-gradient-to-r from-success to-success-600" style={{ width: '92%' }}></div>
            </div>
          </div>
        </div>
      </div>

      {/* 字体测试 */}
      <div className="card space-y-4">
        <h2 className="text-xl font-semibold text-foreground">字体样式</h2>
        <div className="space-y-4">
          <div>
            <h3 className="text-lg font-medium text-foreground mb-2">Sans Serif (Inter)</h3>
            <p className="text-muted">
              这是使用 Inter 字体的文本，适用于界面文字和标题。
            </p>
          </div>
          <div>
            <h3 className="text-lg font-medium text-foreground mb-2">Monospace (JetBrains Mono)</h3>
            <code className="font-mono text-sm bg-surfaceHighlight px-2 py-1 rounded text-primary">
              const cookie = "sessionKey=abc123; userId=456";
            </code>
          </div>
        </div>
      </div>

      {/* 表格测试 */}
      <div className="card space-y-4">
        <h2 className="text-xl font-semibold text-foreground">表格样式</h2>
        <div className="overflow-x-auto">
          <table className="table">
            <thead>
              <tr>
                <th>Cookie ID</th>
                <th>Status</th>
                <th>Requests</th>
                <th>Last Used</th>
              </tr>
            </thead>
            <tbody>
              <tr>
                <td className="font-mono text-sm">abc123...def456</td>
                <td><span className="badge-alive">Alive</span></td>
                <td className="font-mono">42</td>
                <td className="text-muted text-sm">2 minutes ago</td>
              </tr>
              <tr>
                <td className="font-mono text-sm">ghi789...jkl012</td>
                <td><span className="badge-banned">Banned</span></td>
                <td className="font-mono">156</td>
                <td className="text-muted text-sm">1 hour ago</td>
              </tr>
              <tr>
                <td className="font-mono text-sm">mno345...pqr678</td>
                <td><span className="badge-checking">Checking</span></td>
                <td className="font-mono">8</td>
                <td className="text-muted text-sm">Just now</td>
              </tr>
            </tbody>
          </table>
        </div>
      </div>

      {/* 加载动画测试 */}
      <div className="card space-y-4">
        <h2 className="text-xl font-semibold text-foreground">加载动画</h2>
        <div className="flex items-center gap-6">
          <div className="loading-spinner w-8 h-8"></div>
          <div className="loading-spinner w-6 h-6"></div>
          <div className="loading-spinner w-4 h-4"></div>
        </div>
      </div>

      {/* 测试完成提示 */}
      <div className="glass-strong p-6 rounded-xl text-center">
        <h2 className="text-2xl font-bold text-gradient mb-2">
          🎉 主题配置测试完成
        </h2>
        <p className="text-muted">
          如果所有样式都正确显示，说明 Tailwind 配置和赛博朋克主题已成功加载！
        </p>
      </div>
    </div>
  );
};

export default ThemeTest;

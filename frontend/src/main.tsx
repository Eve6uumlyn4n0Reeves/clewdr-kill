// frontend/src/main.tsx
import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import "./index.css";
import App from "./App.tsx";
import { AppProvider } from "./context/AppContext";

// 初始化深色模式和赛博朋克主题
const initializeTheme = () => {
  // 强制启用深色模式
  document.documentElement.classList.add("dark");

  // 设置主题色彩变量
  document.documentElement.style.setProperty("--primary-color", "#8b5cf6");
  document.documentElement.style.setProperty("--success-color", "#10b981");
  document.documentElement.style.setProperty("--danger-color", "#ef4444");
  document.documentElement.style.setProperty("--warning-color", "#f59e0b");
  document.documentElement.style.setProperty("--info-color", "#06b6d4");

  // 设置背景渐变
  document.body.style.background = `
    linear-gradient(135deg, #09090b 0%, #18181b 100%),
    radial-gradient(circle at 20% 80%, rgba(139, 92, 246, 0.05) 0%, transparent 50%),
    radial-gradient(circle at 80% 20%, rgba(16, 185, 129, 0.05) 0%, transparent 50%)
  `;
  document.body.style.backgroundAttachment = "fixed";

  // 添加赛博朋克网格背景
  const meshOverlay = document.createElement("div");
  meshOverlay.className = "fixed inset-0 pointer-events-none z-0 opacity-30";
  meshOverlay.style.backgroundImage = `
    linear-gradient(rgba(139, 92, 246, 0.03) 1px, transparent 1px),
    linear-gradient(90deg, rgba(139, 92, 246, 0.03) 1px, transparent 1px)
  `;
  meshOverlay.style.backgroundSize = "50px 50px";
  document.body.appendChild(meshOverlay);

  // 添加动态光效
  const glowEffect = document.createElement("div");
  glowEffect.className = "fixed inset-0 pointer-events-none z-0";
  glowEffect.style.background = `
    radial-gradient(600px circle at var(--mouse-x, 50%) var(--mouse-y, 50%),
    rgba(139, 92, 246, 0.02), transparent 40%)
  `;
  document.body.appendChild(glowEffect);

  // 鼠标跟踪光效
  let mouseX = 0;
  let mouseY = 0;

  document.addEventListener("mousemove", (e) => {
    mouseX = e.clientX;
    mouseY = e.clientY;

    glowEffect.style.setProperty("--mouse-x", `${mouseX}px`);
    glowEffect.style.setProperty("--mouse-y", `${mouseY}px`);
  });

  // 添加页面加载动画
  document.body.style.opacity = "0";
  document.body.style.transition = "opacity 0.5s ease-in-out";

  // 页面加载完成后显示
  window.addEventListener("load", () => {
    setTimeout(() => {
      document.body.style.opacity = "1";
    }, 100);
  });
};

// 初始化主题
initializeTheme();

// 添加全局键盘快捷键
document.addEventListener("keydown", (e) => {
  // Ctrl/Cmd + K 快速搜索（预留）
  if ((e.ctrlKey || e.metaKey) && e.key === "k") {
    e.preventDefault();
    // 触发搜索功能（后续实现）
    console.log("Quick search triggered");
  }

  // ESC 键关闭模态框（预留）
  if (e.key === "Escape") {
    // 触发关闭模态框功能（后续实现）
    console.log("Escape pressed");
  }
});

// 添加性能监控
if (import.meta.env.DEV) {
  // 开发环境下的性能监控
  const observer = new PerformanceObserver((list) => {
    for (const entry of list.getEntries()) {
      if (entry.entryType === "measure") {
        console.log(`⚡ ${entry.name}: ${entry.duration.toFixed(2)}ms`);
      }
    }
  });
  observer.observe({ entryTypes: ["measure"] });
}

// 错误边界处理
window.addEventListener("error", (e) => {
  console.error("🚨 Global error:", e.error);
});

window.addEventListener("unhandledrejection", (e) => {
  console.error("🚨 Unhandled promise rejection:", e.reason);
});

createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <AppProvider>
      <App />
    </AppProvider>
  </StrictMode>,
);

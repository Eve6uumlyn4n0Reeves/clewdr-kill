/** @type {import('tailwindcss').Config} */
export default {
  content: ["./index.html", "./src/**/*.{js,ts,jsx,tsx}"],
  darkMode: "class",
  theme: {
    extend: {
      colors: {
        // 背景色系：极深色调，减少视觉疲劳
        background: "#09090b", // Zinc-950
        surface: "#18181b", // Zinc-900
        surfaceHighlight: "#27272a", // Zinc-800
        surfaceHover: "#3f3f46", // Zinc-700

        // 边框和分割线
        border: "#27272a",
        borderHighlight: "#3f3f46",

        // 文本色系
        foreground: "#fafafa", // Zinc-50
        muted: "#a1a1aa", // Zinc-400
        mutedForeground: "#71717a", // Zinc-500

        // 主色调：紫色系（科技感）
        primary: {
          DEFAULT: "#8b5cf6", // Violet-500
          50: "#f5f3ff",
          100: "#ede9fe",
          200: "#ddd6fe",
          300: "#c4b5fd",
          400: "#a78bfa",
          500: "#8b5cf6",
          600: "#7c3aed",
          700: "#6d28d9",
          800: "#5b21b6",
          900: "#4c1d95",
          950: "#2e1065",
          glow: "rgba(139, 92, 246, 0.5)",
          glowStrong: "rgba(139, 92, 246, 0.8)",
        },

        // 成功色：绿色系（存活状态）
        success: {
          DEFAULT: "#10b981", // Emerald-500
          50: "#ecfdf5",
          100: "#d1fae5",
          200: "#a7f3d0",
          300: "#6ee7b7",
          400: "#34d399",
          500: "#10b981",
          600: "#059669",
          700: "#047857",
          800: "#065f46",
          900: "#064e3b",
          950: "#022c22",
          glow: "rgba(16, 185, 129, 0.5)",
          glowStrong: "rgba(16, 185, 129, 0.8)",
        },

        // 危险色：红色系（封禁状态）
        danger: {
          DEFAULT: "#ef4444", // Red-500
          50: "#fef2f2",
          100: "#fee2e2",
          200: "#fecaca",
          300: "#fca5a5",
          400: "#f87171",
          500: "#ef4444",
          600: "#dc2626",
          700: "#b91c1c",
          800: "#991b1b",
          900: "#7f1d1d",
          950: "#450a0a",
          glow: "rgba(239, 68, 68, 0.5)",
          glowStrong: "rgba(239, 68, 68, 0.8)",
        },

        // 警告色：琥珀色系（待处理状态）
        warning: {
          DEFAULT: "#f59e0b", // Amber-500
          50: "#fffbeb",
          100: "#fef3c7",
          200: "#fde68a",
          300: "#fcd34d",
          400: "#fbbf24",
          500: "#f59e0b",
          600: "#d97706",
          700: "#b45309",
          800: "#92400e",
          900: "#78350f",
          950: "#451a03",
          glow: "rgba(245, 158, 11, 0.5)",
          glowStrong: "rgba(245, 158, 11, 0.8)",
        },

        // 信息色：青色系（处理中状态）
        info: {
          DEFAULT: "#06b6d4", // Cyan-500
          50: "#ecfeff",
          100: "#cffafe",
          200: "#a5f3fc",
          300: "#67e8f9",
          400: "#22d3ee",
          500: "#06b6d4",
          600: "#0891b2",
          700: "#0e7490",
          800: "#155e75",
          900: "#164e63",
          950: "#083344",
          glow: "rgba(6, 182, 212, 0.5)",
          glowStrong: "rgba(6, 182, 212, 0.8)",
        },

        // 卡片和组件背景
        card: "#18181b",
        cardHover: "#27272a",

        // 输入框
        input: "#27272a",
        inputBorder: "#3f3f46",
        inputFocus: "#8b5cf6",

        // 按钮
        buttonPrimary: "#8b5cf6",
        buttonPrimaryHover: "#7c3aed",
        buttonSecondary: "#27272a",
        buttonSecondaryHover: "#3f3f46",
      },

      fontFamily: {
        // 主字体：Inter 现代无衬线
        sans: [
          "Inter",
          "system-ui",
          "-apple-system",
          "BlinkMacSystemFont",
          "sans-serif",
        ],
        // 等宽字体：用于代码、数据展示
        mono: [
          '"JetBrains Mono"',
          '"Fira Code"',
          "Consolas",
          "Monaco",
          "monospace",
        ],
        // 标题字体：更有科技感
        display: ["Inter", "system-ui", "sans-serif"],
      },

      fontSize: {
        xs: ["0.75rem", { lineHeight: "1rem" }],
        sm: ["0.875rem", { lineHeight: "1.25rem" }],
        base: ["1rem", { lineHeight: "1.5rem" }],
        lg: ["1.125rem", { lineHeight: "1.75rem" }],
        xl: ["1.25rem", { lineHeight: "1.75rem" }],
        "2xl": ["1.5rem", { lineHeight: "2rem" }],
        "3xl": ["1.875rem", { lineHeight: "2.25rem" }],
        "4xl": ["2.25rem", { lineHeight: "2.5rem" }],
        "5xl": ["3rem", { lineHeight: "1" }],
        "6xl": ["3.75rem", { lineHeight: "1" }],
      },

      boxShadow: {
        // 辉光效果
        "glow-sm": "0 0 10px -2px var(--tw-shadow-color)",
        "glow-md": "0 0 20px -4px var(--tw-shadow-color)",
        "glow-lg": "0 0 30px -6px var(--tw-shadow-color)",
        "glow-xl": "0 0 40px -8px var(--tw-shadow-color)",

        // 内阴影效果
        "inner-glow": "inset 0 0 10px -2px var(--tw-shadow-color)",

        // 卡片阴影
        card: "0 4px 6px -1px rgba(0, 0, 0, 0.3), 0 2px 4px -1px rgba(0, 0, 0, 0.2)",
        "card-hover":
          "0 10px 15px -3px rgba(0, 0, 0, 0.4), 0 4px 6px -2px rgba(0, 0, 0, 0.3)",

        // 深色模式专用阴影
        "dark-sm": "0 1px 2px 0 rgba(0, 0, 0, 0.8)",
        "dark-md":
          "0 4px 6px -1px rgba(0, 0, 0, 0.6), 0 2px 4px -1px rgba(0, 0, 0, 0.4)",
        "dark-lg":
          "0 10px 15px -3px rgba(0, 0, 0, 0.6), 0 4px 6px -2px rgba(0, 0, 0, 0.4)",
      },

      animation: {
        // 基础动画
        "fade-in": "fadeIn 0.3s ease-out",
        "fade-out": "fadeOut 0.3s ease-in",
        "slide-up": "slideUp 0.3s ease-out",
        "slide-down": "slideDown 0.3s ease-out",
        "slide-left": "slideLeft 0.3s ease-out",
        "slide-right": "slideRight 0.3s ease-out",
        "scale-in": "scaleIn 0.2s ease-out",
        "scale-out": "scaleOut 0.2s ease-in",

        // 状态指示动画
        "pulse-slow": "pulse 3s cubic-bezier(0.4, 0, 0.6, 1) infinite",
        "pulse-fast": "pulse 1s cubic-bezier(0.4, 0, 0.6, 1) infinite",
        "bounce-subtle": "bounceSubtle 2s infinite",

        // 辉光动画
        "glow-pulse": "glowPulse 2s ease-in-out infinite alternate",
        "glow-rotate": "glowRotate 3s linear infinite",

        // 加载动画
        "spin-slow": "spin 3s linear infinite",
        "ping-slow": "ping 3s cubic-bezier(0, 0, 0.2, 1) infinite",

        // 数据更新动画
        flash: "flash 0.5s ease-in-out",
        highlight: "highlight 1s ease-in-out",
      },

      keyframes: {
        fadeIn: {
          "0%": { opacity: "0" },
          "100%": { opacity: "1" },
        },
        fadeOut: {
          "0%": { opacity: "1" },
          "100%": { opacity: "0" },
        },
        slideUp: {
          "0%": { transform: "translateY(10px)", opacity: "0" },
          "100%": { transform: "translateY(0)", opacity: "1" },
        },
        slideDown: {
          "0%": { transform: "translateY(-10px)", opacity: "0" },
          "100%": { transform: "translateY(0)", opacity: "1" },
        },
        slideLeft: {
          "0%": { transform: "translateX(10px)", opacity: "0" },
          "100%": { transform: "translateX(0)", opacity: "1" },
        },
        slideRight: {
          "0%": { transform: "translateX(-10px)", opacity: "0" },
          "100%": { transform: "translateX(0)", opacity: "1" },
        },
        scaleIn: {
          "0%": { transform: "scale(0.95)", opacity: "0" },
          "100%": { transform: "scale(1)", opacity: "1" },
        },
        scaleOut: {
          "0%": { transform: "scale(1)", opacity: "1" },
          "100%": { transform: "scale(0.95)", opacity: "0" },
        },
        bounceSubtle: {
          "0%, 100%": { transform: "translateY(0)" },
          "50%": { transform: "translateY(-2px)" },
        },
        glowPulse: {
          "0%": { boxShadow: "0 0 5px var(--tw-shadow-color)" },
          "100%": { boxShadow: "0 0 20px var(--tw-shadow-color)" },
        },
        glowRotate: {
          "0%": { transform: "rotate(0deg)" },
          "100%": { transform: "rotate(360deg)" },
        },
        flash: {
          "0%, 100%": { opacity: "1" },
          "50%": { opacity: "0.5" },
        },
        highlight: {
          "0%": { backgroundColor: "transparent" },
          "50%": { backgroundColor: "rgba(139, 92, 246, 0.1)" },
          "100%": { backgroundColor: "transparent" },
        },
      },

      backdropBlur: {
        xs: "2px",
        sm: "4px",
        md: "8px",
        lg: "12px",
        xl: "16px",
        "2xl": "24px",
        "3xl": "32px",
      },

      spacing: {
        18: "4.5rem",
        88: "22rem",
        128: "32rem",
        144: "36rem",
      },

      maxWidth: {
        "8xl": "88rem",
        "9xl": "96rem",
      },

      borderRadius: {
        xl: "0.75rem",
        "2xl": "1rem",
        "3xl": "1.5rem",
      },

      zIndex: {
        60: "60",
        70: "70",
        80: "80",
        90: "90",
        100: "100",
      },
    },
  },
  plugins: [],
};

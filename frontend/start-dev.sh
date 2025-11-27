#!/bin/bash

# ClewdR Kill Edition - 前端开发启动脚本
# 用于快速启动开发环境并验证主题配置

set -e

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# 打印横幅
print_banner() {
    echo -e "${PURPLE}"
    echo "    //   ) )                                    //   ) ) "
    echo "   //        //  ___                   ___   / //___/ /  "
    echo "  //        // //___) ) //  / /  / / //   ) / / ___ (    "
    echo " //        // //       //  / /  / / //   / / //   | |    "
    echo "((____/ / // ((____   ((__( (__/ / ((___/ / //    | |    "
    echo "            KILL EDITION - Frontend Dev"
    echo -e "${NC}"
}

# 检查 Node.js 版本
check_node() {
    echo -e "${CYAN}🔍 检查 Node.js 环境...${NC}"

    if ! command -v node &> /dev/null; then
        echo -e "${RED}❌ Node.js 未安装，请先安装 Node.js 18+${NC}"
        exit 1
    fi

    NODE_VERSION=$(node -v | cut -d'v' -f2 | cut -d'.' -f1)
    if [ "$NODE_VERSION" -lt 18 ]; then
        echo -e "${RED}❌ Node.js 版本过低 (当前: $(node -v))，需要 18+${NC}"
        exit 1
    fi

    echo -e "${GREEN}✅ Node.js 版本: $(node -v)${NC}"
}

# 检查包管理器
check_package_manager() {
    echo -e "${CYAN}🔍 检查包管理器...${NC}"

    if command -v pnpm &> /dev/null; then
        PKG_MANAGER="pnpm"
        echo -e "${GREEN}✅ 使用 pnpm${NC}"
    elif command -v yarn &> /dev/null; then
        PKG_MANAGER="yarn"
        echo -e "${GREEN}✅ 使用 yarn${NC}"
    else
        PKG_MANAGER="npm"
        echo -e "${GREEN}✅ 使用 npm${NC}"
    fi
}

# 安装依赖
install_dependencies() {
    echo -e "${CYAN}📦 安装依赖包...${NC}"

    if [ ! -d "node_modules" ] || [ ! -f "package-lock.json" ]; then
        echo -e "${YELLOW}⚠️  依赖包未安装或不完整，开始安装...${NC}"
        $PKG_MANAGER install
        echo -e "${GREEN}✅ 依赖包安装完成${NC}"
    else
        echo -e "${GREEN}✅ 依赖包已存在${NC}"
    fi
}

# 检查 Tailwind 配置
check_tailwind() {
    echo -e "${CYAN}🎨 检查 Tailwind CSS 配置...${NC}"

    if [ ! -f "tailwind.config.js" ]; then
        echo -e "${RED}❌ tailwind.config.js 未找到${NC}"
        exit 1
    fi

    if [ ! -f "postcss.config.js" ]; then
        echo -e "${RED}❌ postcss.config.js 未找到${NC}"
        exit 1
    fi

    echo -e "${GREEN}✅ Tailwind CSS 配置正常${NC}"
}

# 检查环境变量
check_env() {
    echo -e "${CYAN}🔧 检查环境配置...${NC}"

    if [ ! -f ".env" ]; then
        echo -e "${YELLOW}⚠️  .env 文件不存在，创建默认配置...${NC}"
        cat > .env << EOF
# ClewdR Kill Edition - 前端环境配置
VITE_API_BASE_URL=/api
VITE_APP_TITLE=ClewdR Kill Edition
VITE_APP_VERSION=0.11.27
VITE_THEME=cyberpunk
VITE_DEBUG=true
EOF
        echo -e "${GREEN}✅ 已创建默认 .env 配置${NC}"
    else
        echo -e "${GREEN}✅ 环境配置文件存在${NC}"
    fi
}

# 运行类型检查
run_type_check() {
    echo -e "${CYAN}🔍 运行 TypeScript 类型检查...${NC}"

    if $PKG_MANAGER run type-check 2>/dev/null; then
        echo -e "${GREEN}✅ TypeScript 类型检查通过${NC}"
    else
        echo -e "${YELLOW}⚠️  TypeScript 类型检查有警告，但继续启动...${NC}"
    fi
}

# 启动开发服务器
start_dev_server() {
    echo -e "${CYAN}🚀 启动开发服务器...${NC}"
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${GREEN}🌐 前端地址: http://localhost:5173${NC}"
    echo -e "${GREEN}🎨 主题测试: http://localhost:5173/theme-test${NC}"
    echo -e "${GREEN}📊 控制台: http://localhost:5173 (需要后端运行)${NC}"
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${YELLOW}💡 提示: 按 Ctrl+C 停止服务器${NC}"
    echo -e "${YELLOW}💡 提示: 访问 /theme-test 查看赛博朋克主题效果${NC}"
    echo ""

    # 启动 Vite 开发服务器
    $PKG_MANAGER run dev
}

# 主函数
main() {
    print_banner

    # 检查当前目录
    if [ ! -f "package.json" ]; then
        echo -e "${RED}❌ 请在前端项目根目录运行此脚本${NC}"
        exit 1
    fi

    # 执行检查和启动流程
    check_node
    check_package_manager
    install_dependencies
    check_tailwind
    check_env
    run_type_check

    echo -e "${GREEN}🎉 所有检查通过，准备启动开发服务器...${NC}"
    echo ""

    # 延迟 1 秒让用户看到成功信息
    sleep 1

    start_dev_server
}

# 错误处理
trap 'echo -e "\n${RED}❌ 启动过程中断${NC}"; exit 1' INT TERM

# 运行主函数
main "$@"

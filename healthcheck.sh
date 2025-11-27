#!/bin/sh

# ClewdR Kill Edition - Docker Health Check Script
# 用于Docker容器的健康检查，确保服务正常运行

set -e

# 配置
API_HOST="localhost"
API_PORT="8484"
TIMEOUT=10
MAX_RETRIES=3

# 颜色定义（如果支持）
if [ -t 1 ]; then
    RED='\033[0;31m'
    GREEN='\033[0;32m'
    YELLOW='\033[1;33m'
    NC='\033[0m'
else
    RED=''
    GREEN=''
    YELLOW=''
    NC=''
fi

# 日志函数
log_info() {
    echo "${GREEN}[HEALTH]${NC} $1" >&2
}

log_warn() {
    echo "${YELLOW}[HEALTH]${NC} $1" >&2
}

log_error() {
    echo "${RED}[HEALTH]${NC} $1" >&2
}

# 检查基础连接
check_connection() {
    if ! nc -z "$API_HOST" "$API_PORT" 2>/dev/null; then
        log_error "无法连接到 $API_HOST:$API_PORT"
        return 1
    fi
    return 0
}

# 检查API健康端点
check_api_health() {
    local url="http://$API_HOST:$API_PORT/api/health"

    # 使用curl检查健康端点
    if command -v curl >/dev/null 2>&1; then
        if curl -f -s --max-time "$TIMEOUT" "$url" >/dev/null 2>&1; then
            return 0
        else
            log_error "API健康检查失败: $url"
            return 1
        fi
    fi

    # 如果没有curl，使用wget
    if command -v wget >/dev/null 2>&1; then
        if wget -q --timeout="$TIMEOUT" --tries=1 -O /dev/null "$url" 2>/dev/null; then
            return 0
        else
            log_error "API健康检查失败: $url"
            return 1
        fi
    fi

    log_error "未找到curl或wget命令"
    return 1
}

# 检查API响应内容
check_api_response() {
    local url="http://$API_HOST:$API_PORT/api/health"
    local response

    if command -v curl >/dev/null 2>&1; then
        response=$(curl -f -s --max-time "$TIMEOUT" "$url" 2>/dev/null)
    elif command -v wget >/dev/null 2>&1; then
        response=$(wget -q --timeout="$TIMEOUT" --tries=1 -O - "$url" 2>/dev/null)
    else
        log_warn "无法获取API响应内容"
        return 0  # 不作为失败条件
    fi

    # 检查响应是否包含预期内容
    if echo "$response" | grep -q "status.*ok\|healthy\|success" 2>/dev/null; then
        log_info "API响应正常"
        return 0
    else
        log_warn "API响应内容异常: $response"
        return 1
    fi
}

# 检查进程状态
check_process() {
    # 检查clewdr进程是否运行
    if pgrep -f "clewdr" >/dev/null 2>&1; then
        log_info "ClewdR进程运行正常"
        return 0
    else
        log_error "ClewdR进程未运行"
        return 1
    fi
}

# 检查数据库连接
check_database() {
    local db_path="/app/data/clewdr.db"

    # 检查数据库文件是否存在
    if [ ! -f "$db_path" ]; then
        log_warn "数据库文件不存在: $db_path"
        return 1
    fi

    # 检查数据库是否可读写
    if [ ! -r "$db_path" ] || [ ! -w "$db_path" ]; then
        log_error "数据库文件权限异常: $db_path"
        return 1
    fi

    # 如果有sqlite3命令，尝试简单查询
    if command -v sqlite3 >/dev/null 2>&1; then
        if sqlite3 "$db_path" "SELECT 1;" >/dev/null 2>&1; then
            log_info "数据库连接正常"
            return 0
        else
            log_error "数据库查询失败"
            return 1
        fi
    fi

    log_info "数据库文件检查通过"
    return 0
}

# 检查磁盘空间
check_disk_space() {
    local data_dir="/app/data"
    local min_free_mb=100

    if [ -d "$data_dir" ]; then
        # 获取可用空间（MB）
        local free_space
        free_space=$(df "$data_dir" | awk 'NR==2 {print int($4/1024)}')

        if [ "$free_space" -lt "$min_free_mb" ]; then
            log_error "磁盘空间不足: ${free_space}MB < ${min_free_mb}MB"
            return 1
        else
            log_info "磁盘空间充足: ${free_space}MB"
            return 0
        fi
    fi

    return 0
}

# 检查内存使用
check_memory() {
    # 获取当前进程内存使用（如果可能）
    local pid
    pid=$(pgrep -f "clewdr" | head -1)

    if [ -n "$pid" ] && [ -f "/proc/$pid/status" ]; then
        local mem_kb
        mem_kb=$(awk '/VmRSS:/ {print $2}' "/proc/$pid/status" 2>/dev/null)

        if [ -n "$mem_kb" ]; then
            local mem_mb=$((mem_kb / 1024))
            log_info "内存使用: ${mem_mb}MB"

            # 检查是否超过1GB（可能的内存泄漏）
            if [ "$mem_mb" -gt 1024 ]; then
                log_warn "内存使用较高: ${mem_mb}MB"
            fi
        fi
    fi

    return 0
}

# 主健康检查函数
main_health_check() {
    local checks_passed=0
    local total_checks=0

    log_info "开始健康检查..."

    # 基础连接检查
    total_checks=$((total_checks + 1))
    if check_connection; then
        checks_passed=$((checks_passed + 1))
    fi

    # API健康检查
    total_checks=$((total_checks + 1))
    if check_api_health; then
        checks_passed=$((checks_passed + 1))
    fi

    # API响应检查
    total_checks=$((total_checks + 1))
    if check_api_response; then
        checks_passed=$((checks_passed + 1))
    fi

    # 进程检查
    total_checks=$((total_checks + 1))
    if check_process; then
        checks_passed=$((checks_passed + 1))
    fi

    # 数据库检查
    total_checks=$((total_checks + 1))
    if check_database; then
        checks_passed=$((checks_passed + 1))
    fi

    # 磁盘空间检查
    total_checks=$((total_checks + 1))
    if check_disk_space; then
        checks_passed=$((checks_passed + 1))
    fi

    # 内存检查（仅警告，不影响健康状态）
    check_memory

    # 评估健康状态
    local success_rate=$((checks_passed * 100 / total_checks))

    log_info "健康检查完成: $checks_passed/$total_checks 通过 (${success_rate}%)"

    # 至少80%的检查通过才认为健康
    if [ "$success_rate" -ge 80 ]; then
        log_info "服务状态: 健康"
        return 0
    else
        log_error "服务状态: 不健康"
        return 1
    fi
}

# 重试机制
health_check_with_retry() {
    local attempt=1

    while [ $attempt -le $MAX_RETRIES ]; do
        if [ $attempt -gt 1 ]; then
            log_info "重试健康检查 ($attempt/$MAX_RETRIES)..."
            sleep 2
        fi

        if main_health_check; then
            return 0
        fi

        attempt=$((attempt + 1))
    done

    log_error "健康检查失败，已重试 $MAX_RETRIES 次"
    return 1
}

# 快速检查模式（仅检查API）
quick_check() {
    log_info "快速健康检查..."

    if check_connection && check_api_health; then
        log_info "快速检查通过"
        return 0
    else
        log_error "快速检查失败"
        return 1
    fi
}

# 详细检查模式
detailed_check() {
    log_info "详细健康检查..."

    # 显示系统信息
    echo "=== 系统信息 ==="
    echo "时间: $(date)"
    echo "主机: $(hostname)"
    echo "负载: $(uptime | awk -F'load average:' '{print $2}' | xargs)"

    if [ -f /proc/meminfo ]; then
        echo "内存: $(awk '/MemAvailable:/ {printf "%.1fMB可用", $2/1024}' /proc/meminfo)"
    fi

    echo "=== 服务检查 ==="

    # 执行完整检查
    health_check_with_retry
}

# 解析命令行参数
case "${1:-full}" in
    "quick"|"-q"|"--quick")
        quick_check
        ;;
    "detailed"|"-d"|"--detailed")
        detailed_check
        ;;
    "full"|"-f"|"--full"|"")
        health_check_with_retry
        ;;
    "help"|"-h"|"--help")
        echo "用法: $0 [quick|detailed|full|help]"
        echo ""
        echo "选项:"
        echo "  quick     快速检查（仅API）"
        echo "  detailed  详细检查（包含系统信息）"
        echo "  full      完整检查（默认）"
        echo "  help      显示帮助"
        exit 0
        ;;
    *)
        log_error "未知选项: $1"
        echo "使用 '$0 help' 查看帮助"
        exit 1
        ;;
esac

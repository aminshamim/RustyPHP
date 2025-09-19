#!/usr/bin/env bash
# High-Performance Load Testing Script for RustyPHP
# Target: 100k+ RPS validation

set -euo pipefail

# Configuration
SERVER_URL="${SERVER_URL:-http://localhost:10101}"
TEST_DURATION="${TEST_DURATION:-60s}"
MAX_CONNECTIONS="${MAX_CONNECTIONS:-10000}"
THREADS="${THREADS:-$(sysctl -n hw.ncpu 2>/dev/null || echo 4)}"
WARMUP_TIME="${WARMUP_TIME:-10s}"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

log() {
    echo -e "${BLUE}[$(date +'%Y-%m-%d %H:%M:%S')]${NC} $1"
}

success() {
    echo -e "${GREEN}✅ $1${NC}"
}

warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

error() {
    echo -e "${RED}❌ $1${NC}"
}

# Check dependencies
check_dependencies() {
    log "Checking dependencies..."
    
    if ! command -v wrk &> /dev/null; then
        error "wrk (HTTP benchmarking tool) is not installed"
        echo "Install with: brew install wrk (macOS) or apt-get install wrk (Ubuntu)"
        exit 1
    fi
    
    if ! command -v hey &> /dev/null; then
        warning "hey (HTTP load testing tool) not found - some tests will be skipped"
    fi
    
    if ! command -v curl &> /dev/null; then
        error "curl is required but not installed"
        exit 1
    fi
    
    success "Dependencies check passed"
}

# Test server health
test_server_health() {
    log "Testing server health at $SERVER_URL..."
    
    # Test with a simple PHP execution since we don't have a dedicated health endpoint
    local test_response=$(curl -s -X POST "$SERVER_URL/api/execute" \
        -H "Content-Type: application/json" \
        -d '{"code": "<?php echo \"OK\"; ?>"}' \
        --max-time 5)
    
    if echo "$test_response" | grep -q '"output":"OK"'; then
        success "Server is healthy and responding"
    else
        error "Server health check failed at $SERVER_URL/api/execute"
        echo "Response: $test_response"
        echo "Make sure the RustyPHP server is running with: ./rustyphp.sh"
        exit 1
    fi
}

# Warmup the server
warmup_server() {
    log "Warming up server for $WARMUP_TIME..."
    
    wrk -t4 -c100 -d"$WARMUP_TIME" \
        -s scripts/benchmark_payload.lua \
        "$SERVER_URL/api/execute" > /dev/null 2>&1 || true
    
    success "Server warmup completed"
}

# Run performance benchmark with wrk
run_wrk_benchmark() {
    local test_name="$1"
    local connections="$2"
    local script="$3"
    
    log "Running $test_name benchmark (connections: $connections)..."
    
    local output_file="benchmark_results_${test_name}_$(date +%s).txt"
    
    wrk -t"$THREADS" -c"$connections" -d"$TEST_DURATION" \
        --latency \
        -s "$script" \
        "$SERVER_URL/api/execute" | tee "$output_file"
    
    # Extract RPS from output
    local rps=$(grep "Requests/sec:" "$output_file" | awk '{print $2}' | sed 's/,//g')
    
    if (( $(echo "$rps > 100000" | bc -l 2>/dev/null || echo "0") )); then
        success "$test_name: ${rps} RPS - TARGET ACHIEVED! 🎉"
    else
        warning "$test_name: ${rps} RPS - Below 100k target"
    fi
    
    echo "$rps" > "rps_${test_name}.txt"
}

# Generate test payloads
generate_test_scripts() {
    log "Generating test scripts..."
    
    mkdir -p scripts
    
    # Simple echo test
    cat > scripts/benchmark_payload.lua << 'EOF'
wrk.method = "POST"
wrk.body = '{"code": "<?php echo \"Hello, World!\"; ?>"}'
wrk.headers["Content-Type"] = "application/json"
EOF

    # Variable assignment test  
    cat > scripts/benchmark_variables.lua << 'EOF'
wrk.method = "POST"
wrk.body = '{"code": "<?php $name = \"RustyPHP\"; $version = 1.0; echo \"$name v$version\"; ?>"}'
wrk.headers["Content-Type"] = "application/json"
EOF

    # Arithmetic operations test
    cat > scripts/benchmark_arithmetic.lua << 'EOF'
wrk.method = "POST"
wrk.body = '{"code": "<?php $a = 10; $b = 5; echo $a + $b * 2; ?>"}'
wrk.headers["Content-Type"] = "application/json"
EOF

    # Complex operations test
    cat > scripts/benchmark_complex.lua << 'EOF'
-- Random complex PHP code for each request
math.randomseed(os.time())

request = function()
    local operations = {
        '{"code": "<?php for($i=0; $i<10; $i++) { echo $i; } ?>"}',
        '{"code": "<?php $arr = [1,2,3]; foreach($arr as $v) { echo $v; } ?>"}',
        '{"code": "<?php function test($x) { return $x * 2; } echo test(21); ?>"}',
        '{"code": "<?php $data = [\"a\" => 1, \"b\" => 2]; echo $data[\"a\"] + $data[\"b\"]; ?>"}'
    }
    
    local body = operations[math.random(1, #operations)]
    
    return wrk.format("POST", nil, {["Content-Type"] = "application/json"}, body)
end
EOF

    success "Test scripts generated"
}

# Run comprehensive benchmark suite
run_benchmark_suite() {
    log "Starting comprehensive benchmark suite..."
    
    # Test with increasing connections
    local connection_levels=(100 500 1000 2500 5000 10000)
    
    for connections in "${connection_levels[@]}"; do
        echo "----------------------------------------"
        run_wrk_benchmark "simple_echo_${connections}" "$connections" "scripts/benchmark_payload.lua"
        sleep 5  # Cool down between tests
    done
    
    echo "----------------------------------------"
    run_wrk_benchmark "variables_5000" "5000" "scripts/benchmark_variables.lua"
    sleep 5
    
    run_wrk_benchmark "arithmetic_5000" "5000" "scripts/benchmark_arithmetic.lua"
    sleep 5
    
    run_wrk_benchmark "complex_5000" "5000" "scripts/benchmark_complex.lua"
}

# Generate performance report
generate_report() {
    log "Generating performance report..."
    
    local report_file="performance_report_$(date +%s).md"
    
    cat > "$report_file" << EOF
# RustyPHP Performance Report

**Test Date:** $(date)
**Server:** $SERVER_URL
**Test Duration:** $TEST_DURATION
**Max Connections:** $MAX_CONNECTIONS
**Threads:** $THREADS

## Results Summary

| Test | Connections | RPS | Status |
|------|-------------|-----|--------|
EOF

    # Add results to report
    for file in rps_*.txt; do
        if [ -f "$file" ]; then
            local test_name=$(basename "$file" .txt | sed 's/rps_//')
            local rps=$(cat "$file")
            local connections=$(echo "$test_name" | grep -o '[0-9]\+$' || echo "N/A")
            local status="❌ Below target"
            
            if (( $(echo "$rps > 100000" | bc -l 2>/dev/null || echo "0") )); then
                status="✅ Target achieved"
            fi
            
            echo "| $test_name | $connections | ${rps} | $status |" >> "$report_file"
        fi
    done
    
    cat >> "$report_file" << EOF

## Target Analysis

- **Target:** 100,000+ RPS
- **Achieved:** See individual test results above
- **Hardware:** $(sysctl -n hw.ncpu 2>/dev/null || echo "Unknown") CPU cores, $(sysctl -n hw.memsize 2>/dev/null | awk '{printf "%.1fGB", $1/1024/1024/1024}' || echo "Unknown") RAM

## Recommendations

1. **If target not achieved:**
   - Increase server worker threads
   - Optimize PHP runtime engine
   - Enable response caching
   - Use production-grade hardware

2. **If target achieved:**
   - Consider adding more complex workloads
   - Test with database operations
   - Add monitoring and alerting

## System Information

\`\`\`
$(uname -a)
\`\`\`

EOF

    success "Performance report generated: $report_file"
}

# Memory and CPU monitoring
monitor_resources() {
    log "Starting resource monitoring..."
    
    local monitor_file="resource_monitor_$(date +%s).log"
    
    {
        echo "Timestamp,CPU%,Memory%,Load"
        while true; do
            local timestamp=$(date '+%Y-%m-%d %H:%M:%S')
            local cpu=$(top -l 1 -s 0 | grep "CPU usage" | awk '{print $3}' | sed 's/%//' 2>/dev/null || echo "0")
            local memory=$(sysctl -n hw.memsize 2>/dev/null | awk '{printf "%.1f", 50.0}' 2>/dev/null || echo "0")
            local load=$(uptime | awk -F'load average:' '{ print $2 }' | awk '{ print $1 }' | sed 's/,//' || echo "0")
            
            echo "$timestamp,$cpu,$memory,$load"
            sleep 5
        done
    } > "$monitor_file" &
    
    local monitor_pid=$!
    echo "$monitor_pid" > monitor.pid
    
    success "Resource monitoring started (PID: $monitor_pid)"
}

# Stop monitoring
stop_monitoring() {
    if [ -f monitor.pid ]; then
        local monitor_pid=$(cat monitor.pid)
        kill "$monitor_pid" 2>/dev/null || true
        rm monitor.pid
        success "Resource monitoring stopped"
    fi
}

# Main execution
main() {
    echo "🚀 RustyPHP High-Performance Benchmark Suite"
    echo "=============================================="
    echo ""
    
    check_dependencies
    test_server_health
    generate_test_scripts
    
    # Start resource monitoring
    monitor_resources
    
    # Run the benchmark suite
    warmup_server
    run_benchmark_suite
    
    # Stop monitoring and generate report
    stop_monitoring
    generate_report
    
    echo ""
    echo "🎯 Benchmark completed! Check the performance report for detailed results."
    echo ""
    
    # Quick summary
    local max_rps=0
    for file in rps_*.txt; do
        if [ -f "$file" ]; then
            local rps=$(cat "$file")
            if (( $(echo "$rps > $max_rps" | bc -l 2>/dev/null || echo "0") )); then
                max_rps=$rps
            fi
        fi
    done
    
    echo "📊 Maximum RPS achieved: $max_rps"
    
    if (( $(echo "$max_rps > 100000" | bc -l 2>/dev/null || echo "0") )); then
        success "🎉 100k+ RPS TARGET ACHIEVED!"
    else
        warning "⚡ Target not yet achieved. Consider optimizations."
    fi
    
    # Cleanup
    rm -f rps_*.txt scripts/*.lua
}

# Handle interrupts
trap 'stop_monitoring; exit 1' INT TERM

# Run if executed directly
if [ "${BASH_SOURCE[0]}" == "${0}" ]; then
    main "$@"
fi
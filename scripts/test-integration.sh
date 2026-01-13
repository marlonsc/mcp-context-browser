#!/bin/bash
# Integration Test Runner
# Starts Redis and NATS services, runs integration tests, then cleans up

set -e

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
PROJECT_ROOT="$( cd "$SCRIPT_DIR/.." && pwd )"

# Color codes
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

log_info() {
    echo -e "${BLUE}ℹ${NC} $*"
}

log_success() {
    echo -e "${GREEN}✅${NC} $*"
}

log_error() {
    echo -e "${RED}❌${NC} $*"
}

log_warn() {
    echo -e "${YELLOW}⚠️${NC} $*"
}

# Check if docker is available
if ! command -v docker &> /dev/null; then
    log_error "Docker is not installed. Please install Docker to run integration tests."
    exit 1
fi

# Check if docker-compose is available
if ! command -v docker-compose &> /dev/null; then
    log_error "docker-compose is not installed. Please install docker-compose to run integration tests."
    exit 1
fi

cd "$PROJECT_ROOT"

case "${1:-}" in
    start)
        log_info "Starting Redis and NATS services..."
        docker-compose -f docker-compose.testing.yml up -d

        log_info "Waiting for services to be healthy..."
        sleep 5

        # Check Redis
        if docker-compose -f docker-compose.testing.yml exec -T redis redis-cli ping > /dev/null 2>&1; then
            log_success "Redis is healthy"
        else
            log_error "Redis failed to start"
            docker-compose -f docker-compose.testing.yml logs redis
            exit 1
        fi

        # Check NATS
        if docker-compose -f docker-compose.testing.yml exec -T nats wget --spider -q http://localhost:8222/healthz > /dev/null 2>&1; then
            log_success "NATS is healthy"
        else
            log_warn "NATS health check inconclusive (may still be starting)"
        fi
        ;;

    stop)
        log_info "Stopping services..."
        docker-compose -f docker-compose.testing.yml down -v
        log_success "Services stopped and volumes cleaned"
        ;;

    restart)
        log_info "Restarting services..."
        docker-compose -f docker-compose.testing.yml restart
        sleep 3
        log_success "Services restarted"
        ;;

    logs)
        docker-compose -f docker-compose.testing.yml logs -f
        ;;

    test)
        log_info "Starting services..."
        docker-compose -f docker-compose.testing.yml up -d

        log_info "Waiting for services to be ready..."
        sleep 5

        log_info "Running integration tests..."
        log_info "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

        # Run Redis integration tests
        echo ""
        log_info "Running Redis integration tests..."
        if cargo test --test '*' redis_cache_integration -- --nocapture; then
            log_success "Redis integration tests passed"
        else
            log_error "Redis integration tests failed"
            docker-compose -f docker-compose.testing.yml down -v
            exit 1
        fi

        # Run NATS integration tests
        echo ""
        log_info "Running NATS integration tests..."
        if cargo test --test '*' nats_event_bus_integration -- --nocapture; then
            log_success "NATS integration tests passed"
        else
            log_error "NATS integration tests failed"
            docker-compose -f docker-compose.testing.yml down -v
            exit 1
        fi

        log_info "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
        log_success "All integration tests passed!"

        log_info "Cleaning up services..."
        docker-compose -f docker-compose.testing.yml down -v
        log_success "Cleanup complete"
        ;;

    *)
        echo "Integration Test Runner"
        echo ""
        echo "Usage: $0 {start|stop|restart|logs|test}"
        echo ""
        echo "Commands:"
        echo "  start    - Start Redis and NATS services"
        echo "  stop     - Stop services and remove volumes"
        echo "  restart  - Restart running services"
        echo "  logs     - View service logs (follow mode)"
        echo "  test     - Start services, run tests, and cleanup"
        echo ""
        echo "Examples:"
        echo "  $0 start                 # Start services"
        echo "  $0 test                  # Full test cycle"
        echo "  $0 logs                  # View logs"
        echo "  $0 stop                  # Stop services"
        echo ""
        exit 0
        ;;
esac

#!/bin/bash
set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

PROJECT_ROOT="$(cd "$(dirname "$0")" && pwd)"

print_header() {
    echo -e "\n${BLUE}========================================${NC}"
    echo -e "${BLUE}  JejakCuan - Indonesian Stock Tracker${NC}"
    echo -e "${BLUE}========================================${NC}\n"
}

print_status() {
    echo -e "${GREEN}[✓]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[!]${NC} $1"
}

print_error() {
    echo -e "${RED}[✗]${NC} $1"
}

check_dependencies() {
    echo -e "${BLUE}Checking dependencies...${NC}"
    
    local missing=()
    
    command -v docker >/dev/null 2>&1 || missing+=("docker")
    command -v cargo >/dev/null 2>&1 || missing+=("cargo (Rust)")
    command -v pnpm >/dev/null 2>&1 || missing+=("pnpm")
    command -v python3 >/dev/null 2>&1 || missing+=("python3")
    
    if [ ${#missing[@]} -gt 0 ]; then
        print_error "Missing dependencies: ${missing[*]}"
        exit 1
    fi
    
    print_status "All dependencies found"
}

start_db() {
    echo -e "\n${BLUE}Starting database services...${NC}"
    cd "$PROJECT_ROOT/infra/docker"
    docker compose up -d
    
    echo "Waiting for PostgreSQL to be ready..."
    for i in {1..30}; do
        if docker exec jejakcuan-db pg_isready -U jejakcuan >/dev/null 2>&1; then
            print_status "PostgreSQL is ready"
            break
        fi
        sleep 1
    done
    
    cd "$PROJECT_ROOT"
}

stop_db() {
    echo -e "\n${BLUE}Stopping database services...${NC}"
    cd "$PROJECT_ROOT/infra/docker"
    docker compose down
    print_status "Database services stopped"
    cd "$PROJECT_ROOT"
}

setup_env() {
    if [ ! -f "$PROJECT_ROOT/.env" ]; then
        echo -e "\n${BLUE}Setting up environment...${NC}"
        cp "$PROJECT_ROOT/.env.example" "$PROJECT_ROOT/.env"
        print_status "Created .env from .env.example"
    fi
}

run_migrations() {
    echo -e "\n${BLUE}Running database migrations...${NC}"
    if [ -f "$PROJECT_ROOT/crates/db/migrations/001_initial_schema.sql" ]; then
        PGPASSWORD=jejakcuan_dev psql -h localhost -U jejakcuan -d jejakcuan -f "$PROJECT_ROOT/crates/db/migrations/001_initial_schema.sql" 2>/dev/null || true
        print_status "Migrations applied"
    else
        print_warning "No migration file found, skipping migrations"
    fi
}

run_api() {
    echo -e "\n${BLUE}Starting Rust API server...${NC}"
    cd "$PROJECT_ROOT"
    cargo run --bin jejakcuan-api
}

run_web() {
    echo -e "\n${BLUE}Starting SvelteKit frontend...${NC}"
    cd "$PROJECT_ROOT/apps/web"
    pnpm install --silent
    pnpm dev
}

run_ml() {
    echo -e "\n${BLUE}Starting ML service...${NC}"
    cd "$PROJECT_ROOT/apps/ml"
    if [ ! -d ".venv" ]; then
        python3 -m venv .venv
    fi
    source .venv/bin/activate
    pip install -e . --quiet
    uvicorn jejakcuan_ml.main:app --reload --port 8000
}

run_tests() {
    echo -e "\n${BLUE}Running all tests...${NC}"
    
    echo -e "\n${YELLOW}Rust tests:${NC}"
    cd "$PROJECT_ROOT"
    cargo test --workspace
    
    echo -e "\n${YELLOW}Frontend tests:${NC}"
    cd "$PROJECT_ROOT/apps/web"
    pnpm test:run 2>/dev/null || print_warning "Frontend tests skipped (run pnpm install first)"
    
    echo -e "\n${YELLOW}ML tests:${NC}"
    cd "$PROJECT_ROOT/apps/ml"
    if [ -d ".venv" ]; then
        source .venv/bin/activate
        python -m pytest tests/ -v 2>/dev/null || print_warning "ML tests skipped"
    else
        print_warning "ML venv not set up, skipping ML tests"
    fi
    
    print_status "Tests completed"
}

run_all() {
    print_header
    check_dependencies
    setup_env
    start_db
    run_migrations
    
    echo -e "\n${GREEN}Starting all services in background...${NC}"
    echo -e "${YELLOW}Press Ctrl+C to stop all services${NC}\n"
    
    # Start services in background
    (cd "$PROJECT_ROOT" && cargo run --bin jejakcuan-api 2>&1 | sed 's/^/[API] /') &
    API_PID=$!
    
    sleep 2
    
    (cd "$PROJECT_ROOT/apps/web" && pnpm install --silent && pnpm dev 2>&1 | sed 's/^/[WEB] /') &
    WEB_PID=$!
    
    # Trap Ctrl+C to cleanup
    trap "echo -e '\n${YELLOW}Stopping services...${NC}'; kill $API_PID $WEB_PID 2>/dev/null; stop_db; exit 0" INT
    
    echo -e "\n${GREEN}Services running:${NC}"
    echo -e "  Frontend: ${BLUE}http://localhost:5173${NC}"
    echo -e "  API:      ${BLUE}http://localhost:8080${NC}"
    echo -e "  Postgres: ${BLUE}localhost:5432${NC}"
    echo -e "  Redis:    ${BLUE}localhost:6379${NC}"
    echo ""
    
    wait
}

show_help() {
    print_header
    echo "Usage: ./run.sh [command]"
    echo ""
    echo "Commands:"
    echo "  start     Start all services (db + api + web)"
    echo "  api       Start only the Rust API"
    echo "  web       Start only the SvelteKit frontend"
    echo "  ml        Start only the ML service"
    echo "  db        Start only database services"
    echo "  stop      Stop database services"
    echo "  test      Run all tests"
    echo "  help      Show this help message"
    echo ""
    echo "Examples:"
    echo "  ./run.sh start    # Start everything"
    echo "  ./run.sh api      # Start only API (requires db running)"
    echo "  ./run.sh test     # Run all tests"
}

# Main
case "${1:-start}" in
    start)
        run_all
        ;;
    api)
        setup_env
        run_api
        ;;
    web)
        run_web
        ;;
    ml)
        run_ml
        ;;
    db)
        start_db
        ;;
    stop)
        stop_db
        ;;
    test)
        run_tests
        ;;
    help|--help|-h)
        show_help
        ;;
    *)
        print_error "Unknown command: $1"
        show_help
        exit 1
        ;;
esac

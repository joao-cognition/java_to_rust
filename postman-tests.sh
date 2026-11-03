#!/usr/bin/env bash
# Run the Newman non-regression collection against a PetClinic REST service.
#
#   ./postman-tests.sh [BASE_URL]
#   BASE_URL=http://localhost:9977/petclinic ./postman-tests.sh
#
# BASE_URL is the service root without /api (default: Java on :9966).
# The collection writes data: reset the DB first (./scripts/reset-db.sh).
set -uo pipefail
cd "$(dirname "$0")"

# ------------------- Constants & Config ---------------------------------------------------
BASE_URL="${1:-${BASE_URL:-http://localhost:9966/petclinic}}"
BASE_URL="${BASE_URL%/}"
COLLECTION="./src/test/postman/petclinic-nonregressiontests.postman_collection.json"
ENVIRONMENT="./src/test/postman/petclinic-env.postman_environment.json"
REPORT_DIR="./src/test/postman/reports"

GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[0;33m'
NC='\033[0m'

REQUIRED_CMDS=("node" "npx" "jq")

mkdir -p "$REPORT_DIR"

# --------------------------- Checking required dependencies ------------------------------
for CMD in "${REQUIRED_CMDS[@]}"; do
  if ! command -v "$CMD" &>/dev/null; then
    echo -e "${RED}❌ Error: '$CMD' is required but not installed.${NC}"
    echo -e "${YELLOW}💡 Try installing it with one of the following:${NC}"
    echo -e "${YELLOW}  • brew install $CMD          # macOS${NC}"
    echo -e "${YELLOW}  • sudo apt install $CMD     # Debian/Ubuntu${NC}"
    echo -e "${YELLOW}  • sudo dnf install $CMD     # Fedora${NC}"
    echo -e "${YELLOW}  • sudo pacman -S $CMD       # Arch Linux${NC}"
    exit 1
  fi
done

# -------------- Getting folder (scenarios) names from collection ---------------------------
mapfile -t SCENARIOS < <(jq -r '.item[].name' "$COLLECTION")

if [ ${#SCENARIOS[@]} -eq 0 ]; then
  echo -e "${RED}❌ Error: No folders (scenarios) found in collection.${NC}"
  exit 1
fi

# ---------------- Announce & list scenarios ---------------------------------------------
echo ""
echo -e "${GREEN}🐾 Starting Petclinic API tests with Newman against ${BASE_URL}${NC}"
echo ""
for SCENARIO in "${SCENARIOS[@]}"; do
  echo -e "${YELLOW}🔎 Will execute scenario: $SCENARIO${NC}"
done

# ---------------- Running tests with Newman ------------------------------------
echo ""
echo -e "${GREEN}Running tests...${NC}"
echo ""

npx --yes -p newman -p newman-reporter-htmlextra newman run "$COLLECTION" \
  -e "$ENVIRONMENT" \
  --env-var "baseUrl=$BASE_URL" \
  --reporters cli,htmlextra \
  --reporter-htmlextra-export "$REPORT_DIR/report.html" \
  --timeout-request 5000
EXIT_CODE=$?

# --------------- Checking if the test failed  ----------------------------------------------------
if [ $EXIT_CODE -ne 0 ]; then
  echo -e "${RED}❌ Newman tests failed against ${BASE_URL}. See the HTML report at: $REPORT_DIR/report.html${NC}"
  exit 1
fi

# -------------- Done when is success --------------------------------------------------------------------
echo ""
echo -e "${GREEN}✅ Newman tests passed against ${BASE_URL}${NC}"
echo -e "${YELLOW}📄 HTML report generated at: $REPORT_DIR/report.html${NC}"
echo ""

#!/bin/bash
# Cross-link all integration repos back to the main Plasmate repo
# Run this from the directory containing all cloned integration repos

set -e

BADGE='[![Plasmate](https://img.shields.io/badge/powered%20by-Plasmate-orange)](https://github.com/plasmate-labs/plasmate)'
FOOTER='

---

**Part of the [Plasmate Ecosystem](https://github.com/plasmate-labs/plasmate)** - The browser engine for AI agents. 60+ integrations available.
'

# List of integration repos to update
REPOS=(
  "langchain-plasmate"
  "llamaindex-plasmate"
  "crewai-plasmate"
  "autogen-plasmate"
  "haystack-plasmate"
  "dspy-plasmate"
  "semantic-kernel-plasmate"
  "langflow-plasmate"
  "flowise-plasmate"
  "dify-plasmate"
  "n8n-nodes-plasmate"
  "zapier-plasmate"
  "make-plasmate"
  "activepieces-plasmate"
  "temporal-plasmate"
  "scrapy-plasmate"
  "crawl4ai-plasmate"
  "firecrawl-plasmate"
  "scrapegraphai-plasmate"
  "supabase-plasmate"
  "prisma-plasmate"
  "planetscale-plasmate"
  "airtable-plasmate"
  "vscode-plasmate"
  "cursor-plasmate"
  "raycast-plasmate"
  "copilot-plasmate"
  "openwebui-plasmate"
  "openai-gpt-plasmate"
)

echo "Plasmate Integration Cross-Linker"
echo "================================="
echo ""

# Check if gh CLI is available
if ! command -v gh &> /dev/null; then
  echo "Error: GitHub CLI (gh) is required. Install with: brew install gh"
  exit 1
fi

# Function to add badge and footer to a repo's README
update_readme() {
  local repo=$1
  local readme="$repo/README.md"

  if [[ ! -f "$readme" ]]; then
    echo "  Skipping $repo - no README.md found"
    return
  fi

  # Check if already has the badge
  if grep -q "plasmate-labs/plasmate" "$readme"; then
    echo "  $repo - already cross-linked"
    return
  fi

  # Add badge after first heading
  sed -i '' '1,/^#/{/^#/a\
'"$BADGE"'
}' "$readme"

  # Add footer if not present
  if ! grep -q "Part of the \[Plasmate Ecosystem\]" "$readme"; then
    echo "$FOOTER" >> "$readme"
  fi

  echo "  $repo - updated"
}

# Clone and update, or update if already cloned
for repo in "${REPOS[@]}"; do
  echo "Processing $repo..."

  if [[ -d "$repo" ]]; then
    update_readme "$repo"
  else
    echo "  Cloning $repo..."
    gh repo clone "plasmate-labs/$repo" 2>/dev/null || {
      echo "  Could not clone $repo - may not exist yet"
      continue
    }
    update_readme "$repo"
  fi
done

echo ""
echo "Done! Review changes, then commit and push each repo."
echo ""
echo "Quick push script:"
echo '  for repo in */; do'
echo '    cd "$repo"'
echo '    git add README.md && git commit -m "Add Plasmate ecosystem cross-link" && git push'
echo '    cd ..'
echo '  done'

# Run: vhs issues_machine.tape
Output apps/landing/public/assets/issues_machine.webm
Set FontSize 14
Set Width 800
Set Height 500
Set Padding 20
Set Theme "Catppuccin Mocha"

# Setup
Hide
Type "rm -rf .chisel"
Enter
Type "chisel init"
Enter
Sleep 1s
# Clear default issues and DB
Type "rm .chisel/issues/*.md .chisel/index.db"
Enter
# Copy demo issue
Type "cp demo_data/issue_1.md .chisel/issues/0001_fix.md"
Enter
# Re-index to update DB
Type "chisel issues index"
Enter
Sleep 1s
Type "clear"
Enter
Show

# 1. List initial state
Type "chisel issues list --machine"
Sleep 500ms
Enter
Sleep 2s

# 2. Update status via CLI
# Note: ID is 1 because we cleared defaults and added one file
Type "chisel issues update-status 1 in-progress --machine"
Sleep 500ms
Enter
Sleep 2s

# 3. Verify final state
Type "chisel issues list --machine"
Sleep 500ms
Enter
Sleep 4s

# Run: vhs issues_human.tape
Output apps/landing/public/assets/issues_human.webm
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

# Open Kanban
Type "chisel issues"
Sleep 500ms
Enter
Sleep 2s

# Move status
Type "m"
Sleep 800ms
# Select 'In Progress'
Down
Sleep 300ms
Enter
Sleep 2s

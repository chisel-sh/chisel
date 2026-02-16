# Run: vhs docs_human.tape
Output apps/landing/public/assets/docs_human.webm
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
# Clear defaults
Type "rm -rf .chisel/docs/*"
Enter
# Create directory
Type "mkdir -p .chisel/docs/incidents"
Enter
# Copy demo doc
Type "cp demo_data/doc_1.md .chisel/docs/incidents/postmortem.md"
Enter
Sleep 1s
Type "clear"
Enter
Show

# Open Docs
Type "chisel docs"
Sleep 500ms
Enter
Sleep 2s

# Navigate
Type "j"
Sleep 500ms
Enter
Sleep 1s

# View Content
Type "l"
Sleep 1s
Type "jjjjjjjj"
Sleep 2s
